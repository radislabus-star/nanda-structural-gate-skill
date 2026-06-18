//! Fixed-size contracts for the planned NANDA-6M packed hot core.
//!
//! This module intentionally contains no JSON parsing, strings, maps, or heap
//! containers. It is the byte-level contract that the dynamic CLI layer must
//! pack into before a real cache-resident core can run.

#![allow(dead_code)]

pub const VERSION: &str = "nanda-6m-v12-hot-support-bucket-core";
pub const WAVE_DIM: usize = 1024;

pub const BUDGET_BYTES: usize = 6_291_456;
pub const HEADER_BYTES: usize = 16_384;
pub const TRIAD_ARENA_BYTES: usize = 2_097_152;
pub const CENTROID_ARENA_BYTES: usize = 2_097_152;
pub const LANE_ARENA_BYTES: usize = 1_048_576;
pub const WORKSPACE_BYTES: usize = 786_432;
pub const INDEX_STATS_BYTES: usize = 245_760;

pub const TRIAD_BYTES: usize = 32;
pub const CENTROID_BYTES: usize = 1024;
pub const LANE_BYTES: usize = 64;
pub const QUERY_WAVE_BYTES: usize = WAVE_DIM * core::mem::size_of::<i16>();
pub const PACKED_MIN_FOCUS_SCORE: f64 = 0.01;
pub const PACKED_MIN_FOCUS_MARGIN: f64 = 0.003;
pub const LANE_FOCUSED_NET_DOT: i64 = 128;
pub const LANE_FOCUSED_DELTA_DOT: i64 = 64;

pub const TRIAD_CAPACITY: usize = TRIAD_ARENA_BYTES / TRIAD_BYTES;
pub const CENTROID_CAPACITY: usize = CENTROID_ARENA_BYTES / CENTROID_BYTES;
pub const LANE_CAPACITY: usize = LANE_ARENA_BYTES / LANE_BYTES;
pub const SCORE_BUCKET_CAPACITY: usize = CENTROID_CAPACITY;

pub const RESERVED_CORE_BYTES: usize = HEADER_BYTES
    + TRIAD_ARENA_BYTES
    + CENTROID_ARENA_BYTES
    + LANE_ARENA_BYTES
    + WORKSPACE_BYTES
    + INDEX_STATS_BYTES;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedTriad32 {
    pub subject_id: u32,
    pub object_id: u32,
    pub evidence_ref: u32,
    pub wave_seed: u32,
    pub relation_id: u16,
    pub route_id: u16,
    pub group_id: u16,
    pub role_pack: u16,
    pub flags: u16,
    pub lane_hint: u16,
    pub check: u16,
    pub confidence: u8,
    pub polarity: u8,
}

impl PackedTriad32 {
    pub const fn new(input: PackedTriadInput) -> Self {
        Self {
            subject_id: input.subject_id,
            object_id: input.object_id,
            evidence_ref: input.evidence_ref,
            wave_seed: input.wave_seed,
            relation_id: input.relation_id,
            route_id: input.route_id,
            group_id: input.group_id,
            role_pack: input.role_pack,
            flags: input.flags,
            lane_hint: input.lane_hint,
            check: input.check,
            confidence: input.confidence,
            polarity: input.polarity,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedTriadInput {
    pub subject_id: u32,
    pub object_id: u32,
    pub evidence_ref: u32,
    pub wave_seed: u32,
    pub relation_id: u16,
    pub route_id: u16,
    pub group_id: u16,
    pub role_pack: u16,
    pub flags: u16,
    pub lane_hint: u16,
    pub check: u16,
    pub confidence: u8,
    pub polarity: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedCentroid1024 {
    pub values: [i8; WAVE_DIM],
}

impl Default for PackedCentroid1024 {
    fn default() -> Self {
        Self {
            values: [0; WAVE_DIM],
        }
    }
}

impl PackedCentroid1024 {
    pub fn from_wave(wave: &PackedWave1024, count: usize) -> Self {
        let divisor = count.max(1) as i32;
        let mut centroid = Self::default();
        for (out, value) in centroid.values.iter_mut().zip(wave.values.iter()) {
            let averaged = (*value as i32 / divisor).clamp(i8::MIN as i32, i8::MAX as i32);
            *out = averaged as i8;
        }
        centroid
    }

    pub fn summary(&self) -> CentroidSummary {
        let mut l1: u64 = 0;
        let mut energy: u64 = 0;
        let mut nonzero: usize = 0;
        let mut max_abs: i8 = 0;
        for value in self.values {
            let abs = value.saturating_abs();
            if abs != 0 {
                nonzero += 1;
            }
            max_abs = max_abs.max(abs);
            l1 += abs as u64;
            energy += (value as i64 * value as i64) as u64;
        }
        CentroidSummary {
            l1,
            energy,
            nonzero,
            max_abs,
        }
    }
}

impl PackedWave1024 {
    pub fn score_centroid(&self, centroid: &PackedCentroid1024) -> PeakScore {
        let mut dot: i64 = 0;
        let mut query_energy: i64 = 0;
        let mut centroid_energy: i64 = 0;
        for (query, center) in self.values.iter().zip(centroid.values.iter()) {
            let query = i64::from(*query);
            let center = i64::from(*center);
            dot += query * center;
            query_energy += query * query;
            centroid_energy += center * center;
        }
        let denom = ((query_energy as f64).sqrt() * (centroid_energy as f64).sqrt()).max(1.0);
        PeakScore {
            dot,
            query_energy,
            centroid_energy,
            cosine: dot as f64 / denom,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedLane64 {
    pub support_mask_a: u64,
    pub support_mask_b: u64,
    pub anti_mask_a: u64,
    pub anti_mask_b: u64,
    pub lane_id: u32,
    pub target_route: u16,
    pub target_group: u16,
    pub target_relation: u16,
    pub accepted_count: u16,
    pub rejected_count: u16,
    pub margin_hint: i16,
    pub action: u8,
    pub strength: u8,
    pub reserved: [u8; 14],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedSupportField {
    pub top_id: u16,
    pub key_hash: u32,
    pub positive_dot: i64,
    pub negative_dot: i64,
    pub support_mask_a: u64,
    pub support_mask_b: u64,
    pub anti_mask_a: u64,
    pub anti_mask_b: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PackedAxis {
    #[default]
    Route,
    Group,
}

impl PackedAxis {
    pub const fn triad_id(self, triad: PackedTriad32) -> u16 {
        match self {
            Self::Route => triad.route_id,
            Self::Group => triad.group_id,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PackedAxisPeakState {
    Found = 1,
    Thin = 2,
    Contested = 3,
    #[default]
    NoPeak = 4,
}

impl PackedAxisPeakState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Found => "PEAK_FOUND",
            Self::Thin => "PEAK_THIN",
            Self::Contested => "PEAK_CONTESTED",
            Self::NoPeak => "NO_PEAK",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PackedAxisPeak {
    pub top_id: u16,
    pub top_score: f64,
    pub second_id: u16,
    pub second_score: f64,
    pub margin: f64,
    pub state: PackedAxisPeakState,
}

impl PackedAxisPeak {
    pub fn evaluate(top_id: u16, top_score: f64, second_id: u16, second_score: f64) -> Self {
        let margin = top_score - second_score;
        let state = if top_id == 0 || top_score <= 0.0 {
            PackedAxisPeakState::NoPeak
        } else if top_score < PACKED_MIN_FOCUS_SCORE {
            PackedAxisPeakState::Thin
        } else if margin < PACKED_MIN_FOCUS_MARGIN {
            PackedAxisPeakState::Contested
        } else {
            PackedAxisPeakState::Found
        };
        Self {
            top_id,
            top_score,
            second_id,
            second_score,
            margin,
            state,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PackedDecisionState {
    Focused = 1,
    Thin = 2,
    Contested = 3,
    NoPeak = 4,
    EmptyMemory = 5,
    MemoryFallback = 6,
    EmptyQuery = 7,
    #[default]
    Review = 255,
}

impl PackedDecisionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Focused => "PACKED_FOCUSED",
            Self::Thin => "PACKED_THIN",
            Self::Contested => "PACKED_CONTESTED",
            Self::NoPeak => "PACKED_NO_PEAK",
            Self::EmptyMemory => "PACKED_EMPTY_MEMORY",
            Self::MemoryFallback => "PACKED_MEMORY_FALLBACK",
            Self::EmptyQuery => "PACKED_EMPTY_QUERY",
            Self::Review => "PACKED_REVIEW_REQUIRED",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PackedPeakDecision {
    pub state: PackedDecisionState,
    pub verdict_pass: bool,
    pub safe_to_answer: bool,
    pub query_energy: u64,
    pub memory_records: u16,
    pub query_records: u16,
    pub route: PackedAxisPeak,
    pub group: PackedAxisPeak,
}

impl PackedPeakDecision {
    pub const fn verdict_str(self) -> &'static str {
        if self.verdict_pass {
            "PASS"
        } else {
            "WATCH"
        }
    }

    pub const fn reason(self) -> &'static str {
        match self.state {
            PackedDecisionState::EmptyMemory => {
                "No memory/source triads were packed, so centroids cannot be trusted."
            }
            PackedDecisionState::MemoryFallback => {
                "No candidate/query triads were packed; projection uses memory fallback for diagnostics only."
            }
            PackedDecisionState::EmptyQuery => "Candidate/query projection has zero energy.",
            PackedDecisionState::NoPeak => "At least one centroid axis has no positive peak.",
            PackedDecisionState::Thin => {
                "A peak exists, but cosine strength is below the packed focus threshold."
            }
            PackedDecisionState::Contested => "Top centroid is too close to the runner-up.",
            PackedDecisionState::Focused => "Route and group axes both expose strong packed peaks.",
            PackedDecisionState::Review => "Packed peak decision requires review.",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedSupportSummary {
    pub field: PackedSupportField,
    pub considered: u16,
    pub support_count: u16,
    pub anti_count: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedTriadSupportScore {
    pub route_id: u16,
    pub group_id: u16,
    pub record_index: u16,
    pub dot: i64,
}

impl PackedTriadSupportScore {
    pub const fn axis_id(self, axis: PackedAxis) -> u16 {
        match axis {
            PackedAxis::Route => self.route_id,
            PackedAxis::Group => self.group_id,
        }
    }
}

impl PackedSupportField {
    pub const fn before_net_dot(self) -> i64 {
        self.positive_dot + self.negative_dot
    }

    pub const fn has_anti_support(self) -> bool {
        self.negative_dot < 0 && (self.anti_mask_a != 0 || self.anti_mask_b != 0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedLaneApplication {
    pub lane: PackedLane64,
    pub lane_ready: bool,
    pub improved: bool,
    pub focused_candidate: bool,
    pub before_net_dot: i64,
    pub after_net_dot: i64,
    pub delta_dot: i64,
    pub suppressed_negative_dot: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedLaneSweep {
    pub fields: usize,
    pub lanes: usize,
    pub applied: usize,
    pub improved: usize,
    pub focused: usize,
    pub best_index: u16,
    pub best_after_net_dot: i64,
    pub best_delta_dot: i64,
    pub total_delta_dot: i64,
    pub checksum: u64,
}

pub fn compile_suppress_anti_lane(field: PackedSupportField) -> PackedLane64 {
    let has_lane = field.has_anti_support();
    PackedLane64 {
        support_mask_a: field.anti_mask_a,
        support_mask_b: field.anti_mask_b,
        anti_mask_a: field.support_mask_a,
        anti_mask_b: field.support_mask_b,
        lane_id: field.key_hash,
        target_route: field.top_id,
        target_group: field.top_id,
        target_relation: 0,
        accepted_count: 0,
        rejected_count: if has_lane { 1 } else { 0 },
        margin_hint: field
            .before_net_dot()
            .clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16,
        action: if has_lane { 1 } else { 0 },
        strength: if has_lane { 255 } else { 0 },
        reserved: [0; 14],
    }
}

pub fn apply_suppress_anti_lane(
    field: PackedSupportField,
    lane: PackedLane64,
) -> PackedLaneApplication {
    let before = field.before_net_dot();
    let lane_ready = lane.action == 1
        && lane.strength > 0
        && (lane.support_mask_a != 0 || lane.support_mask_b != 0);
    let after = if lane_ready {
        field.positive_dot
    } else {
        before
    };
    let delta = after - before;
    let improved = lane_ready && delta > 0 && after > before;
    let focused_candidate =
        lane_ready && after >= LANE_FOCUSED_NET_DOT && delta >= LANE_FOCUSED_DELTA_DOT;
    PackedLaneApplication {
        lane,
        lane_ready,
        improved,
        focused_candidate,
        before_net_dot: before,
        after_net_dot: after,
        delta_dot: delta,
        suppressed_negative_dot: if lane_ready { field.negative_dot } else { 0 },
    }
}

pub fn compile_and_apply_suppress_anti_lane(field: PackedSupportField) -> PackedLaneApplication {
    apply_suppress_anti_lane(field, compile_suppress_anti_lane(field))
}

pub fn evaluate_packed_peak_decision(
    route: PackedAxisPeak,
    group: PackedAxisPeak,
    query_energy: u64,
    memory_count: usize,
    query_count: usize,
) -> PackedPeakDecision {
    let memory_records = memory_count.min(usize::from(u16::MAX)) as u16;
    let query_records = query_count.min(usize::from(u16::MAX)) as u16;
    let state = if memory_count == 0 {
        PackedDecisionState::EmptyMemory
    } else if query_count == 0 {
        PackedDecisionState::MemoryFallback
    } else if query_energy == 0 {
        PackedDecisionState::EmptyQuery
    } else if route.state == PackedAxisPeakState::NoPeak
        || group.state == PackedAxisPeakState::NoPeak
    {
        PackedDecisionState::NoPeak
    } else if route.state == PackedAxisPeakState::Thin || group.state == PackedAxisPeakState::Thin {
        PackedDecisionState::Thin
    } else if route.state == PackedAxisPeakState::Contested
        || group.state == PackedAxisPeakState::Contested
    {
        PackedDecisionState::Contested
    } else {
        PackedDecisionState::Focused
    };
    let verdict_pass = state == PackedDecisionState::Focused;
    PackedPeakDecision {
        state,
        verdict_pass,
        safe_to_answer: verdict_pass,
        query_energy,
        memory_records,
        query_records,
        route,
        group,
    }
}

pub fn build_packed_support_field(
    memory: &[PackedTriad32],
    query: &PackedWave1024,
    axis: PackedAxis,
    top_id: u16,
    key_hash: u32,
) -> PackedSupportSummary {
    let mut field = PackedSupportField {
        top_id,
        key_hash,
        ..PackedSupportField::default()
    };
    let mut considered = 0u16;
    let mut support_count = 0u16;
    let mut anti_count = 0u16;
    if top_id == 0 {
        return PackedSupportSummary {
            field,
            considered,
            support_count,
            anti_count,
        };
    }
    let query_energy = query.energy_i64();
    for (index, triad) in memory.iter().copied().enumerate() {
        if axis.triad_id(triad) != top_id {
            continue;
        }
        considered = considered.saturating_add(1);
        let score = score_triad_projection_with_query_energy(query, &triad, query_energy);
        if score.dot > 0 {
            field.positive_dot += score.dot;
            support_count = support_count.saturating_add(1);
            set_support_mask(&mut field.support_mask_a, &mut field.support_mask_b, index);
        } else if score.dot < 0 {
            field.negative_dot += score.dot;
            anti_count = anti_count.saturating_add(1);
            set_support_mask(&mut field.anti_mask_a, &mut field.anti_mask_b, index);
        }
    }
    PackedSupportSummary {
        field,
        considered,
        support_count,
        anti_count,
    }
}

pub fn build_packed_triad_support_scores(
    memory: &[PackedTriad32],
    query: &PackedWave1024,
    scores_out: &mut [PackedTriadSupportScore],
) -> usize {
    let count = memory.len().min(scores_out.len());
    let query_energy = query.energy_i64();
    for (index, (triad, out)) in memory
        .iter()
        .copied()
        .take(count)
        .zip(scores_out.iter_mut())
        .enumerate()
    {
        let score = score_triad_projection_with_query_energy(query, &triad, query_energy);
        *out = PackedTriadSupportScore {
            route_id: triad.route_id,
            group_id: triad.group_id,
            record_index: index.min(usize::from(u16::MAX)) as u16,
            dot: score.dot,
        };
    }
    count
}

pub fn build_packed_support_field_from_scores(
    scores: &[PackedTriadSupportScore],
    axis: PackedAxis,
    top_id: u16,
    key_hash: u32,
) -> PackedSupportSummary {
    let mut field = PackedSupportField {
        top_id,
        key_hash,
        ..PackedSupportField::default()
    };
    let mut considered = 0u16;
    let mut support_count = 0u16;
    let mut anti_count = 0u16;
    if top_id == 0 {
        return PackedSupportSummary {
            field,
            considered,
            support_count,
            anti_count,
        };
    }
    for score in scores.iter().copied() {
        if score.axis_id(axis) != top_id {
            continue;
        }
        considered = considered.saturating_add(1);
        if score.dot > 0 {
            field.positive_dot += score.dot;
            support_count = support_count.saturating_add(1);
            set_support_mask(
                &mut field.support_mask_a,
                &mut field.support_mask_b,
                usize::from(score.record_index),
            );
        } else if score.dot < 0 {
            field.negative_dot += score.dot;
            anti_count = anti_count.saturating_add(1);
            set_support_mask(
                &mut field.anti_mask_a,
                &mut field.anti_mask_b,
                usize::from(score.record_index),
            );
        }
    }
    PackedSupportSummary {
        field,
        considered,
        support_count,
        anti_count,
    }
}

pub fn bucket_packed_triad_support_scores(
    scores: &[PackedTriadSupportScore],
    axis: PackedAxis,
    sorted_out: &mut [PackedTriadSupportScore],
    offsets_out: &mut [u16],
    cursors_out: &mut [u16],
) -> usize {
    let hard_bucket_count = SCORE_BUCKET_CAPACITY
        .min(offsets_out.len().saturating_sub(1))
        .min(cursors_out.len());
    if hard_bucket_count == 0 {
        return 0;
    }
    let max_id = scores
        .iter()
        .map(|score| usize::from(score.axis_id(axis)))
        .filter(|id| *id < hard_bucket_count)
        .max()
        .unwrap_or(0);
    let bucket_count = (max_id + 1).min(hard_bucket_count);
    offsets_out[..=bucket_count].fill(0);
    cursors_out[..bucket_count].fill(0);

    let mut accepted = 0usize;
    for score in scores.iter().copied() {
        let id = usize::from(score.axis_id(axis));
        if id >= bucket_count || accepted >= sorted_out.len() {
            continue;
        }
        cursors_out[id] = cursors_out[id].saturating_add(1);
        accepted += 1;
    }

    let mut running = 0u16;
    for idx in 0..bucket_count {
        offsets_out[idx] = running;
        running = running.saturating_add(cursors_out[idx]);
        cursors_out[idx] = offsets_out[idx];
    }
    offsets_out[bucket_count] = running;
    if bucket_count + 1 < offsets_out.len() {
        offsets_out[bucket_count + 1..].fill(running);
    }

    for score in scores.iter().copied() {
        let id = usize::from(score.axis_id(axis));
        if id >= bucket_count {
            continue;
        }
        let pos = usize::from(cursors_out[id]);
        if pos >= sorted_out.len() || pos >= accepted {
            continue;
        }
        sorted_out[pos] = score;
        cursors_out[id] = cursors_out[id].saturating_add(1);
    }
    accepted
}

pub fn build_packed_support_field_from_score_bucket(
    sorted_scores: &[PackedTriadSupportScore],
    offsets: &[u16],
    axis: PackedAxis,
    top_id: u16,
    key_hash: u32,
) -> PackedSupportSummary {
    let id = usize::from(top_id);
    if top_id == 0 || id + 1 >= offsets.len() {
        return PackedSupportSummary {
            field: PackedSupportField {
                top_id,
                key_hash,
                ..PackedSupportField::default()
            },
            ..PackedSupportSummary::default()
        };
    }
    let start = usize::from(offsets[id]).min(sorted_scores.len());
    let end = usize::from(offsets[id + 1]).min(sorted_scores.len());
    if end <= start {
        return PackedSupportSummary {
            field: PackedSupportField {
                top_id,
                key_hash,
                ..PackedSupportField::default()
            },
            ..PackedSupportSummary::default()
        };
    }
    build_packed_support_field_from_scores(&sorted_scores[start..end], axis, top_id, key_hash)
}

pub fn compile_suppress_anti_lanes(
    fields: &[PackedSupportField],
    lanes_out: &mut [PackedLane64],
) -> usize {
    let count = fields.len().min(lanes_out.len());
    for (field, out) in fields.iter().take(count).zip(lanes_out.iter_mut()) {
        *out = compile_suppress_anti_lane(*field);
    }
    count
}

pub fn apply_suppress_anti_lane_sweep(
    fields: &[PackedSupportField],
    lanes: &[PackedLane64],
) -> PackedLaneSweep {
    let mut sweep = PackedLaneSweep {
        fields: fields.len(),
        lanes: lanes.len(),
        best_after_net_dot: i64::MIN,
        ..PackedLaneSweep::default()
    };
    for (idx, field) in fields.iter().enumerate() {
        let lane = matching_lane(*field, lanes).unwrap_or_default();
        let applied = apply_suppress_anti_lane(*field, lane);
        accumulate_lane_application(&mut sweep, idx, applied);
    }
    if fields.is_empty() {
        sweep.best_after_net_dot = 0;
    }
    sweep
}

pub fn apply_aligned_suppress_anti_lane_sweep(
    fields: &[PackedSupportField],
    lanes: &[PackedLane64],
) -> PackedLaneSweep {
    let count = fields.len().min(lanes.len());
    let mut sweep = PackedLaneSweep {
        fields: fields.len(),
        lanes: lanes.len(),
        best_after_net_dot: i64::MIN,
        ..PackedLaneSweep::default()
    };
    for idx in 0..count {
        let applied = apply_suppress_anti_lane(fields[idx], lanes[idx]);
        accumulate_lane_application(&mut sweep, idx, applied);
    }
    if count == 0 {
        sweep.best_after_net_dot = 0;
    }
    sweep
}

pub fn compile_and_apply_aligned_suppress_anti_lane_sweep(
    fields: &[PackedSupportField],
    lanes_out: &mut [PackedLane64],
) -> PackedLaneSweep {
    let count = fields.len().min(lanes_out.len());
    let mut sweep = PackedLaneSweep {
        fields: fields.len(),
        lanes: lanes_out.len(),
        best_after_net_dot: i64::MIN,
        ..PackedLaneSweep::default()
    };
    for idx in 0..count {
        let lane = compile_suppress_anti_lane(fields[idx]);
        lanes_out[idx] = lane;
        let applied = apply_suppress_anti_lane(fields[idx], lane);
        accumulate_lane_application(&mut sweep, idx, applied);
    }
    if count == 0 {
        sweep.best_after_net_dot = 0;
    }
    sweep
}

fn matching_lane(field: PackedSupportField, lanes: &[PackedLane64]) -> Option<PackedLane64> {
    lanes
        .iter()
        .copied()
        .find(|lane| lane.lane_id == field.key_hash && lane.target_route == field.top_id)
}

fn accumulate_lane_application(
    sweep: &mut PackedLaneSweep,
    idx: usize,
    applied: PackedLaneApplication,
) {
    if applied.lane_ready {
        sweep.applied += 1;
    }
    if applied.improved {
        sweep.improved += 1;
    }
    if applied.focused_candidate {
        sweep.focused += 1;
    }
    if applied.after_net_dot > sweep.best_after_net_dot
        || (applied.after_net_dot == sweep.best_after_net_dot
            && applied.delta_dot > sweep.best_delta_dot)
    {
        sweep.best_index = idx.min(usize::from(u16::MAX)) as u16;
        sweep.best_after_net_dot = applied.after_net_dot;
        sweep.best_delta_dot = applied.delta_dot;
    }
    sweep.total_delta_dot += applied.delta_dot;
    sweep.checksum = sweep
        .checksum
        .wrapping_add(applied.after_net_dot as u64)
        .wrapping_add(applied.delta_dot as u64)
        .wrapping_add(applied.lane.lane_id as u64)
        .wrapping_add(u64::from(applied.focused_candidate));
}

fn set_support_mask(mask_a: &mut u64, mask_b: &mut u64, index: usize) {
    if index < 64 {
        *mask_a |= 1u64 << index;
    } else if index < 128 {
        *mask_b |= 1u64 << (index - 64);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetUsage {
    pub active_triads: usize,
    pub centroids: usize,
    pub lanes: usize,
}

impl BudgetUsage {
    pub const fn estimated_hot_bytes(self) -> usize {
        HEADER_BYTES
            + (self.active_triads * TRIAD_BYTES)
            + (self.centroids * CENTROID_BYTES)
            + (self.lanes * LANE_BYTES)
            + WORKSPACE_BYTES
            + INDEX_STATS_BYTES
    }

    pub const fn fits(self) -> bool {
        self.active_triads <= TRIAD_CAPACITY
            && self.centroids <= CENTROID_CAPACITY
            && self.lanes <= LANE_CAPACITY
            && self.estimated_hot_bytes() <= BUDGET_BYTES
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RawPeakState {
    Focused = 1,
    Thin = 2,
    Contested = 3,
    NoPeak = 4,
    #[default]
    Review = 255,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReplayComputeState {
    Ready = 1,
    Weak = 2,
    #[default]
    None = 0,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReplayFieldState {
    Focused = 1,
    Improved = 2,
    Weakened = 3,
    #[default]
    Observed = 0,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReplayStabilityState {
    StableUnderSoftTouch = 1,
    FullTouchRequired = 2,
    WeakConstructive = 3,
    Destabilizing = 4,
    NoShift = 5,
    #[default]
    NoReplayField = 0,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReplayVerdict {
    StableWithReplay = 1,
    ReplayRescuedThinField = 2,
    ReplayDestabilizedField = 3,
    ReplayTooStrongRequired = 4,
    ReplayComputeReadyReview = 5,
    ReplayWeakOrAmbiguous = 6,
    #[default]
    NoReplayEvidence = 0,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReplayAction {
    KeepGateDecision = 1,
    ReviewReplayRescuedField = 2,
    StopRepairOrSplit = 3,
    ReviewInterventionDependence = 4,
    ReviewReplayEffect = 5,
    #[default]
    UseRawDecision = 0,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReplayTouch {
    pub after_net_dot: i64,
    pub delta_dot: i64,
    pub field_state: ReplayFieldState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayDecisionInput {
    pub raw_state: RawPeakState,
    pub raw_safe_to_answer: bool,
    pub raw_verdict_pass: bool,
    pub matched_keys: u64,
    pub observer_net_dot: i64,
    pub full_delta_dot: i64,
    pub soft: ReplayTouch,
    pub full: ReplayTouch,
    pub stability_state: ReplayStabilityState,
    pub compute_state: ReplayComputeState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayDecision {
    pub verdict: ReplayVerdict,
    pub action: ReplayAction,
    pub output_pass: bool,
    pub output_veto: bool,
    pub safe_to_answer: bool,
    pub observer_net_dot: i64,
    pub soft_touch_net_dot: i64,
    pub full_touch_net_dot: i64,
    pub full_delta_dot: i64,
    pub matched_keys: u64,
}

pub fn evaluate_replay(input: ReplayDecisionInput) -> ReplayDecision {
    let verdict = if input.matched_keys == 0 {
        ReplayVerdict::NoReplayEvidence
    } else if input.stability_state == ReplayStabilityState::Destabilizing
        || input.full.field_state == ReplayFieldState::Weakened
    {
        ReplayVerdict::ReplayDestabilizedField
    } else if input.raw_state == RawPeakState::Thin
        && input.soft.field_state == ReplayFieldState::Focused
    {
        ReplayVerdict::ReplayRescuedThinField
    } else if input.raw_safe_to_answer
        && input.stability_state == ReplayStabilityState::StableUnderSoftTouch
    {
        ReplayVerdict::StableWithReplay
    } else if input.stability_state == ReplayStabilityState::FullTouchRequired {
        ReplayVerdict::ReplayTooStrongRequired
    } else if input.compute_state == ReplayComputeState::Ready {
        ReplayVerdict::ReplayComputeReadyReview
    } else {
        ReplayVerdict::ReplayWeakOrAmbiguous
    };

    let action = match verdict {
        ReplayVerdict::StableWithReplay => ReplayAction::KeepGateDecision,
        ReplayVerdict::ReplayRescuedThinField => ReplayAction::ReviewReplayRescuedField,
        ReplayVerdict::ReplayDestabilizedField => ReplayAction::StopRepairOrSplit,
        ReplayVerdict::ReplayTooStrongRequired => ReplayAction::ReviewInterventionDependence,
        ReplayVerdict::NoReplayEvidence => ReplayAction::UseRawDecision,
        ReplayVerdict::ReplayComputeReadyReview | ReplayVerdict::ReplayWeakOrAmbiguous => {
            ReplayAction::ReviewReplayEffect
        }
    };

    ReplayDecision {
        verdict,
        action,
        output_pass: verdict == ReplayVerdict::StableWithReplay && input.raw_safe_to_answer,
        output_veto: verdict == ReplayVerdict::ReplayDestabilizedField,
        safe_to_answer: false,
        observer_net_dot: input.observer_net_dot,
        soft_touch_net_dot: input.soft.after_net_dot,
        full_touch_net_dot: input.full.after_net_dot,
        full_delta_dot: input.full_delta_dot,
        matched_keys: input.matched_keys,
    }
}

impl ReplayVerdict {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StableWithReplay => "STABLE_WITH_REPLAY",
            Self::ReplayRescuedThinField => "REPLAY_RESCUED_THIN_FIELD",
            Self::ReplayDestabilizedField => "REPLAY_DESTABILIZED_FIELD",
            Self::ReplayTooStrongRequired => "REPLAY_TOO_STRONG_REQUIRED",
            Self::ReplayComputeReadyReview => "REPLAY_COMPUTE_READY_REVIEW",
            Self::ReplayWeakOrAmbiguous => "REPLAY_WEAK_OR_AMBIGUOUS",
            Self::NoReplayEvidence => "NO_REPLAY_EVIDENCE",
        }
    }
}

impl ReplayAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::KeepGateDecision => "KEEP_GATE_DECISION",
            Self::ReviewReplayRescuedField => "REVIEW_REPLAY_RESCUED_FIELD",
            Self::StopRepairOrSplit => "STOP_REPAIR_OR_SPLIT",
            Self::ReviewInterventionDependence => "REVIEW_INTERVENTION_DEPENDENCE",
            Self::ReviewReplayEffect => "REVIEW_REPLAY_EFFECT",
            Self::UseRawDecision => "USE_RAW_DECISION",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedWave1024 {
    pub values: [i16; WAVE_DIM],
}

impl Default for PackedWave1024 {
    fn default() -> Self {
        Self {
            values: [0; WAVE_DIM],
        }
    }
}

impl PackedWave1024 {
    pub fn accumulate_triad(&mut self, triad: &PackedTriad32) {
        let mut state = projection_seed(triad);
        let strength = projection_strength(triad);
        for value in &mut self.values {
            state = mix64(state);
            let sign = if (state & 1) == 0 {
                -strength
            } else {
                strength
            };
            *value = value.saturating_add(sign);
        }
    }

    pub fn from_triads(triads: &[PackedTriad32]) -> Self {
        let mut wave = Self::default();
        for triad in triads {
            wave.accumulate_triad(triad);
        }
        wave
    }

    pub fn summary(&self) -> WaveSummary {
        let mut l1: u64 = 0;
        let mut energy: u64 = 0;
        let mut nonzero: usize = 0;
        let mut max_abs: i16 = 0;
        for value in self.values {
            let abs = value.saturating_abs();
            if abs != 0 {
                nonzero += 1;
            }
            max_abs = max_abs.max(abs);
            l1 += abs as u64;
            energy += (value as i64 * value as i64) as u64;
        }
        WaveSummary {
            l1,
            energy,
            nonzero,
            max_abs,
        }
    }

    pub fn energy_i64(&self) -> i64 {
        self.values
            .iter()
            .map(|value| {
                let value = i64::from(*value);
                value * value
            })
            .sum()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WaveSummary {
    pub l1: u64,
    pub energy: u64,
    pub nonzero: usize,
    pub max_abs: i16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CentroidSummary {
    pub l1: u64,
    pub energy: u64,
    pub nonzero: usize,
    pub max_abs: i8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PeakScore {
    pub dot: i64,
    pub query_energy: i64,
    pub centroid_energy: i64,
    pub cosine: f64,
}

pub fn project_triads(triads: &[PackedTriad32]) -> PackedWave1024 {
    PackedWave1024::from_triads(triads)
}

pub fn centroid_from_triads(triads: &[PackedTriad32]) -> PackedCentroid1024 {
    let wave = project_triads(triads);
    PackedCentroid1024::from_wave(&wave, triads.len())
}

pub fn score_centroid(query: &PackedWave1024, centroid: &PackedCentroid1024) -> PeakScore {
    query.score_centroid(centroid)
}

pub fn score_triad_projection(query: &PackedWave1024, triad: &PackedTriad32) -> PeakScore {
    score_triad_projection_with_query_energy(query, triad, query.energy_i64())
}

pub fn score_triad_projection_with_query_energy(
    query: &PackedWave1024,
    triad: &PackedTriad32,
    query_energy: i64,
) -> PeakScore {
    let mut state = projection_seed(triad);
    let strength = i64::from(projection_strength(triad));
    let mut dot: i64 = 0;
    let mut triad_energy: i64 = 0;
    for query_value in query.values {
        state = mix64(state);
        let projected = if (state & 1) == 0 {
            -strength
        } else {
            strength
        };
        let query_value = i64::from(query_value);
        dot += query_value * projected;
        triad_energy += projected * projected;
    }
    let denom = ((query_energy as f64).sqrt() * (triad_energy as f64).sqrt()).max(1.0);
    PeakScore {
        dot,
        query_energy,
        centroid_energy: triad_energy,
        cosine: dot as f64 / denom,
    }
}

fn projection_seed(triad: &PackedTriad32) -> u64 {
    let mut state = u64::from(triad.wave_seed) << 32 | u64::from(triad.check);
    state ^= u64::from(triad.subject_id).rotate_left(7);
    state ^= u64::from(triad.object_id).rotate_left(17);
    state ^= u64::from(triad.evidence_ref).rotate_left(31);
    state ^= u64::from(triad.relation_id) << 3;
    state ^= u64::from(triad.route_id) << 19;
    state ^= u64::from(triad.group_id) << 37;
    state ^= u64::from(triad.role_pack) << 43;
    state ^= u64::from(triad.flags) << 53;
    mix64(state)
}

fn projection_strength(triad: &PackedTriad32) -> i16 {
    let base = 1 + i16::from(triad.confidence / 64);
    if triad.polarity & 1 == 0 {
        base
    } else {
        -base
    }
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_record_sizes_match_contract() {
        assert_eq!(core::mem::size_of::<PackedTriad32>(), TRIAD_BYTES);
        assert_eq!(core::mem::size_of::<PackedCentroid1024>(), CENTROID_BYTES);
        assert_eq!(core::mem::size_of::<PackedLane64>(), LANE_BYTES);
        assert_eq!(core::mem::size_of::<PackedWave1024>(), QUERY_WAVE_BYTES);
    }

    #[test]
    fn total_budget_is_exactly_six_mib() {
        assert_eq!(RESERVED_CORE_BYTES, BUDGET_BYTES);
        assert_eq!(TRIAD_CAPACITY, 65_536);
        assert_eq!(CENTROID_CAPACITY, 2_048);
        assert_eq!(LANE_CAPACITY, 16_384);
    }

    #[test]
    fn focused_usage_fits_and_overflow_refuses() {
        assert!(BudgetUsage {
            active_triads: 10,
            centroids: 8,
            lanes: 0,
        }
        .fits());

        assert!(!BudgetUsage {
            active_triads: TRIAD_CAPACITY + 1,
            centroids: 1,
            lanes: 0,
        }
        .fits());
    }

    #[test]
    fn packed_projection_is_deterministic_and_nonzero() {
        let triad = PackedTriad32::new(PackedTriadInput {
            subject_id: 1,
            object_id: 2,
            evidence_ref: 3,
            wave_seed: 4,
            relation_id: 5,
            route_id: 6,
            group_id: 7,
            role_pack: 0x0201,
            flags: 1,
            lane_hint: 0,
            check: 8,
            confidence: 230,
            polarity: 9,
        });
        let left = project_triads(&[triad]);
        let right = project_triads(&[triad]);
        assert_eq!(left, right);
        let summary = left.summary();
        assert_eq!(summary.nonzero, WAVE_DIM);
        assert!(summary.energy > 0);
        assert!(summary.max_abs > 0);
    }

    #[test]
    fn packed_centroid_is_quantized_and_nonzero() {
        let triads = [
            PackedTriad32::new(PackedTriadInput {
                subject_id: 1,
                object_id: 2,
                evidence_ref: 3,
                wave_seed: 4,
                relation_id: 5,
                route_id: 6,
                group_id: 7,
                role_pack: 0x0201,
                flags: 1,
                lane_hint: 0,
                check: 8,
                confidence: 230,
                polarity: 9,
            }),
            PackedTriad32::new(PackedTriadInput {
                subject_id: 2,
                object_id: 3,
                evidence_ref: 4,
                wave_seed: 9,
                relation_id: 5,
                route_id: 6,
                group_id: 7,
                role_pack: 0x0201,
                flags: 1,
                lane_hint: 0,
                check: 10,
                confidence: 230,
                polarity: 9,
            }),
        ];
        let centroid = centroid_from_triads(&triads);
        let summary = centroid.summary();
        assert!(summary.nonzero > 0);
        assert!(summary.energy > 0);
        assert!(summary.max_abs > 0);
    }

    #[test]
    fn packed_peak_score_prefers_matching_centroid() {
        let triad_a = PackedTriad32::new(PackedTriadInput {
            subject_id: 1,
            object_id: 2,
            evidence_ref: 3,
            wave_seed: 4,
            relation_id: 5,
            route_id: 6,
            group_id: 7,
            role_pack: 0x0201,
            flags: 1,
            lane_hint: 0,
            check: 8,
            confidence: 230,
            polarity: 9,
        });
        let triad_b = PackedTriad32::new(PackedTriadInput {
            subject_id: 100,
            object_id: 200,
            evidence_ref: 300,
            wave_seed: 400,
            relation_id: 50,
            route_id: 60,
            group_id: 70,
            role_pack: 0x0201,
            flags: 1,
            lane_hint: 0,
            check: 80,
            confidence: 230,
            polarity: 9,
        });
        let query = project_triads(&[triad_a]);
        let matching = centroid_from_triads(&[triad_a]);
        let foreign = centroid_from_triads(&[triad_b]);
        let matching_score = score_centroid(&query, &matching);
        let foreign_score = score_centroid(&query, &foreign);
        assert!(matching_score.cosine > foreign_score.cosine);
        assert!(matching_score.cosine > 0.5);
    }

    #[test]
    fn direct_triad_projection_score_matches_single_triad_centroid() {
        let triad = PackedTriad32::new(PackedTriadInput {
            subject_id: 1,
            object_id: 2,
            evidence_ref: 3,
            wave_seed: 4,
            relation_id: 5,
            route_id: 6,
            group_id: 7,
            role_pack: 0x0201,
            flags: 1,
            lane_hint: 0,
            check: 8,
            confidence: 230,
            polarity: 9,
        });
        let query = project_triads(&[triad]);
        let centroid = centroid_from_triads(&[triad]);
        let centroid_score = score_centroid(&query, &centroid);
        let direct_score = score_triad_projection(&query, &triad);

        assert_eq!(direct_score.dot, centroid_score.dot);
        assert_eq!(direct_score.query_energy, centroid_score.query_energy);
        assert_eq!(direct_score.centroid_energy, centroid_score.centroid_energy);
        assert!((direct_score.cosine - centroid_score.cosine).abs() < f64::EPSILON);
    }

    #[test]
    fn packed_axis_peak_and_decision_are_typed() {
        let route = PackedAxisPeak::evaluate(3, 0.02, 2, 0.01);
        let group = PackedAxisPeak::evaluate(4, 0.03, 5, 0.01);
        let decision = evaluate_packed_peak_decision(route, group, 128, 4, 2);

        assert_eq!(route.state, PackedAxisPeakState::Found);
        assert_eq!(route.state.as_str(), "PEAK_FOUND");
        assert_eq!(decision.state, PackedDecisionState::Focused);
        assert_eq!(decision.state.as_str(), "PACKED_FOCUSED");
        assert_eq!(decision.verdict_str(), "PASS");
        assert!(decision.safe_to_answer);

        let thin = PackedAxisPeak::evaluate(3, 0.001, 2, 0.0);
        let thin_decision = evaluate_packed_peak_decision(thin, group, 128, 4, 2);
        assert_eq!(thin_decision.state, PackedDecisionState::Thin);
        assert_eq!(thin_decision.verdict_str(), "WATCH");
        assert!(!thin_decision.safe_to_answer);
    }

    #[test]
    fn packed_support_builder_splits_positive_and_anti_support() {
        let support = PackedTriad32::new(PackedTriadInput {
            subject_id: 1,
            object_id: 2,
            evidence_ref: 3,
            wave_seed: 4,
            relation_id: 5,
            route_id: 7,
            group_id: 9,
            role_pack: 0x0201,
            flags: 1,
            lane_hint: 0,
            check: 8,
            confidence: 230,
            polarity: 0,
        });
        let anti = PackedTriad32::new(PackedTriadInput {
            polarity: 1,
            ..PackedTriadInput {
                subject_id: 1,
                object_id: 2,
                evidence_ref: 3,
                wave_seed: 4,
                relation_id: 5,
                route_id: 7,
                group_id: 9,
                role_pack: 0x0201,
                flags: 1,
                lane_hint: 0,
                check: 8,
                confidence: 230,
                polarity: 0,
            }
        });
        let foreign = PackedTriad32::new(PackedTriadInput {
            route_id: 11,
            ..PackedTriadInput {
                subject_id: 1,
                object_id: 2,
                evidence_ref: 5,
                wave_seed: 9,
                relation_id: 5,
                route_id: 7,
                group_id: 9,
                role_pack: 0x0201,
                flags: 1,
                lane_hint: 0,
                check: 8,
                confidence: 230,
                polarity: 0,
            }
        });
        let query = project_triads(&[support]);
        let summary = build_packed_support_field(
            &[support, anti, foreign],
            &query,
            PackedAxis::Route,
            7,
            0x55,
        );

        assert_eq!(summary.field.top_id, 7);
        assert_eq!(summary.field.key_hash, 0x55);
        assert_eq!(summary.considered, 2);
        assert_eq!(summary.support_count, 1);
        assert_eq!(summary.anti_count, 1);
        assert!(summary.field.positive_dot > 0);
        assert!(summary.field.negative_dot < 0);
        assert_eq!(summary.field.support_mask_a & 0b001, 0b001);
        assert_eq!(summary.field.anti_mask_a & 0b010, 0b010);
    }

    #[test]
    fn packed_support_score_cache_matches_direct_builder() {
        let support = PackedTriad32::new(PackedTriadInput {
            subject_id: 1,
            object_id: 2,
            evidence_ref: 3,
            wave_seed: 4,
            relation_id: 5,
            route_id: 7,
            group_id: 9,
            role_pack: 0x0201,
            flags: 1,
            lane_hint: 0,
            check: 8,
            confidence: 230,
            polarity: 0,
        });
        let anti = PackedTriad32::new(PackedTriadInput {
            polarity: 1,
            ..PackedTriadInput {
                subject_id: 1,
                object_id: 2,
                evidence_ref: 3,
                wave_seed: 4,
                relation_id: 5,
                route_id: 7,
                group_id: 9,
                role_pack: 0x0201,
                flags: 1,
                lane_hint: 0,
                check: 8,
                confidence: 230,
                polarity: 0,
            }
        });
        let memory = [support, anti];
        let query = project_triads(&[support]);
        let direct = build_packed_support_field(&memory, &query, PackedAxis::Route, 7, 0x55);
        let mut scores = [PackedTriadSupportScore::default(); 2];
        let count = build_packed_triad_support_scores(&memory, &query, &mut scores);
        let cached =
            build_packed_support_field_from_scores(&scores[..count], PackedAxis::Route, 7, 0x55);

        assert_eq!(count, 2);
        assert_eq!(cached, direct);
    }

    #[test]
    fn packed_support_score_buckets_match_score_scan() {
        let support = PackedTriad32::new(PackedTriadInput {
            subject_id: 1,
            object_id: 2,
            evidence_ref: 3,
            wave_seed: 4,
            relation_id: 5,
            route_id: 7,
            group_id: 9,
            role_pack: 0x0201,
            flags: 1,
            lane_hint: 0,
            check: 8,
            confidence: 230,
            polarity: 0,
        });
        let anti = PackedTriad32::new(PackedTriadInput {
            polarity: 1,
            ..PackedTriadInput {
                subject_id: 1,
                object_id: 2,
                evidence_ref: 3,
                wave_seed: 4,
                relation_id: 5,
                route_id: 7,
                group_id: 9,
                role_pack: 0x0201,
                flags: 1,
                lane_hint: 0,
                check: 8,
                confidence: 230,
                polarity: 0,
            }
        });
        let foreign = PackedTriad32::new(PackedTriadInput {
            route_id: 11,
            group_id: 12,
            ..PackedTriadInput {
                subject_id: 1,
                object_id: 2,
                evidence_ref: 5,
                wave_seed: 9,
                relation_id: 5,
                route_id: 7,
                group_id: 9,
                role_pack: 0x0201,
                flags: 1,
                lane_hint: 0,
                check: 8,
                confidence: 230,
                polarity: 0,
            }
        });
        let memory = [support, anti, foreign];
        let query = project_triads(&[support]);
        let mut scores = [PackedTriadSupportScore::default(); 3];
        let count = build_packed_triad_support_scores(&memory, &query, &mut scores);
        let mut sorted = [PackedTriadSupportScore::default(); 3];
        let mut offsets = [0u16; SCORE_BUCKET_CAPACITY + 1];
        let mut cursors = [0u16; SCORE_BUCKET_CAPACITY];

        let accepted = bucket_packed_triad_support_scores(
            &scores[..count],
            PackedAxis::Route,
            &mut sorted,
            &mut offsets,
            &mut cursors,
        );
        let scanned =
            build_packed_support_field_from_scores(&scores[..count], PackedAxis::Route, 7, 0x55);
        let bucketed = build_packed_support_field_from_score_bucket(
            &sorted[..accepted],
            &offsets,
            PackedAxis::Route,
            7,
            0x55,
        );

        assert_eq!(accepted, 3);
        assert_eq!(bucketed, scanned);

        let accepted = bucket_packed_triad_support_scores(
            &scores[..count],
            PackedAxis::Group,
            &mut sorted,
            &mut offsets,
            &mut cursors,
        );
        let scanned =
            build_packed_support_field_from_scores(&scores[..count], PackedAxis::Group, 9, 0x66);
        let bucketed = build_packed_support_field_from_score_bucket(
            &sorted[..accepted],
            &offsets,
            PackedAxis::Group,
            9,
            0x66,
        );

        assert_eq!(accepted, 3);
        assert_eq!(bucketed, scanned);
    }

    #[test]
    fn packed_lane_application_suppresses_anti_support() {
        let field = PackedSupportField {
            top_id: 7,
            key_hash: 0x1234,
            positive_dot: 256,
            negative_dot: -160,
            support_mask_a: 0b0011,
            support_mask_b: 0,
            anti_mask_a: 0b0100,
            anti_mask_b: 0,
        };

        let applied = compile_and_apply_suppress_anti_lane(field);

        assert!(applied.lane_ready);
        assert!(applied.improved);
        assert!(applied.focused_candidate);
        assert_eq!(applied.before_net_dot, 96);
        assert_eq!(applied.after_net_dot, 256);
        assert_eq!(applied.delta_dot, 160);
        assert_eq!(applied.suppressed_negative_dot, -160);
        assert_eq!(applied.lane.support_mask_a, field.anti_mask_a);
        assert_eq!(applied.lane.anti_mask_a, field.support_mask_a);
        assert_eq!(applied.lane.action, 1);
        assert_eq!(applied.lane.strength, 255);
    }

    #[test]
    fn packed_lane_application_noops_without_anti_support() {
        let field = PackedSupportField {
            top_id: 7,
            key_hash: 0x1234,
            positive_dot: 256,
            negative_dot: 0,
            support_mask_a: 0b0011,
            support_mask_b: 0,
            anti_mask_a: 0,
            anti_mask_b: 0,
        };

        let applied = compile_and_apply_suppress_anti_lane(field);

        assert!(!applied.lane_ready);
        assert!(!applied.improved);
        assert!(!applied.focused_candidate);
        assert_eq!(applied.before_net_dot, 256);
        assert_eq!(applied.after_net_dot, 256);
        assert_eq!(applied.delta_dot, 0);
        assert_eq!(applied.suppressed_negative_dot, 0);
        assert_eq!(applied.lane.action, 0);
        assert_eq!(applied.lane.strength, 0);
    }

    #[test]
    fn packed_lane_sweep_applies_matching_lanes() {
        let fields = [
            PackedSupportField {
                top_id: 1,
                key_hash: 0x1001,
                positive_dot: 288,
                negative_dot: -256,
                support_mask_a: 0b0001,
                support_mask_b: 0,
                anti_mask_a: 0b0110,
                anti_mask_b: 0,
            },
            PackedSupportField {
                top_id: 2,
                key_hash: 0x1002,
                positive_dot: 96,
                negative_dot: -32,
                support_mask_a: 0b1000,
                support_mask_b: 0,
                anti_mask_a: 0b0010,
                anti_mask_b: 0,
            },
            PackedSupportField {
                top_id: 3,
                key_hash: 0x1003,
                positive_dot: 256,
                negative_dot: 0,
                support_mask_a: 0b1111,
                support_mask_b: 0,
                anti_mask_a: 0,
                anti_mask_b: 0,
            },
        ];
        let mut lanes = [PackedLane64::default(); 3];
        let compiled = compile_suppress_anti_lanes(&fields, &mut lanes);

        let sweep = apply_suppress_anti_lane_sweep(&fields, &lanes);

        assert_eq!(compiled, 3);
        assert_eq!(sweep.fields, 3);
        assert_eq!(sweep.lanes, 3);
        assert_eq!(sweep.applied, 2);
        assert_eq!(sweep.improved, 2);
        assert_eq!(sweep.focused, 1);
        assert_eq!(sweep.best_index, 0);
        assert_eq!(sweep.best_after_net_dot, 288);
        assert_eq!(sweep.best_delta_dot, 256);
        assert_eq!(sweep.total_delta_dot, 288);
        assert!(sweep.checksum > 0);
    }

    #[test]
    fn aligned_packed_lane_sweep_matches_search_sweep() {
        let fields = [
            PackedSupportField {
                top_id: 1,
                key_hash: 0x1001,
                positive_dot: 288,
                negative_dot: -256,
                support_mask_a: 0b0001,
                support_mask_b: 0,
                anti_mask_a: 0b0110,
                anti_mask_b: 0,
            },
            PackedSupportField {
                top_id: 2,
                key_hash: 0x1002,
                positive_dot: 192,
                negative_dot: -64,
                support_mask_a: 0b1000,
                support_mask_b: 0,
                anti_mask_a: 0b0010,
                anti_mask_b: 0,
            },
        ];
        let mut lanes = [PackedLane64::default(); 2];
        compile_suppress_anti_lanes(&fields, &mut lanes);

        let search_sweep = apply_suppress_anti_lane_sweep(&fields, &lanes);
        let aligned_sweep = apply_aligned_suppress_anti_lane_sweep(&fields, &lanes);

        assert_eq!(aligned_sweep, search_sweep);
        assert_eq!(aligned_sweep.applied, 2);
        assert_eq!(aligned_sweep.focused, 2);
        assert_eq!(aligned_sweep.best_index, 0);
    }

    #[test]
    fn compile_and_apply_aligned_sweep_matches_two_pass_path() {
        let fields = [
            PackedSupportField {
                top_id: 1,
                key_hash: 0x1001,
                positive_dot: 288,
                negative_dot: -256,
                support_mask_a: 0b0001,
                support_mask_b: 0,
                anti_mask_a: 0b0110,
                anti_mask_b: 0,
            },
            PackedSupportField {
                top_id: 2,
                key_hash: 0x1002,
                positive_dot: 192,
                negative_dot: -64,
                support_mask_a: 0b1000,
                support_mask_b: 0,
                anti_mask_a: 0b0010,
                anti_mask_b: 0,
            },
        ];
        let mut two_pass_lanes = [PackedLane64::default(); 2];
        let mut fused_lanes = [PackedLane64::default(); 2];
        compile_suppress_anti_lanes(&fields, &mut two_pass_lanes);

        let two_pass = apply_aligned_suppress_anti_lane_sweep(&fields, &two_pass_lanes);
        let fused = compile_and_apply_aligned_suppress_anti_lane_sweep(&fields, &mut fused_lanes);

        assert_eq!(fused, two_pass);
        assert_eq!(fused_lanes, two_pass_lanes);
        assert_eq!(fused.applied, 2);
        assert_eq!(fused.focused, 2);
    }

    #[test]
    fn packed_lane_sweep_noops_without_matching_lanes() {
        let fields = [PackedSupportField {
            top_id: 1,
            key_hash: 0x1001,
            positive_dot: 288,
            negative_dot: -256,
            support_mask_a: 0b0001,
            support_mask_b: 0,
            anti_mask_a: 0b0110,
            anti_mask_b: 0,
        }];
        let lanes = [PackedLane64 {
            lane_id: 0x9999,
            target_route: 1,
            ..PackedLane64::default()
        }];

        let sweep = apply_suppress_anti_lane_sweep(&fields, &lanes);

        assert_eq!(sweep.applied, 0);
        assert_eq!(sweep.improved, 0);
        assert_eq!(sweep.focused, 0);
        assert_eq!(sweep.best_after_net_dot, 32);
        assert_eq!(sweep.best_delta_dot, 0);
        assert_eq!(sweep.total_delta_dot, 0);
    }

    fn replay_input(
        raw_state: RawPeakState,
        raw_safe_to_answer: bool,
        matched_keys: u64,
        soft: ReplayTouch,
        full: ReplayTouch,
        stability_state: ReplayStabilityState,
        compute_state: ReplayComputeState,
    ) -> ReplayDecisionInput {
        ReplayDecisionInput {
            raw_state,
            raw_safe_to_answer,
            raw_verdict_pass: raw_safe_to_answer,
            matched_keys,
            observer_net_dot: 64,
            full_delta_dot: full.delta_dot,
            soft,
            full,
            stability_state,
            compute_state,
        }
    }

    #[test]
    fn replay_core_reports_no_replay_evidence() {
        let decision = evaluate_replay(replay_input(
            RawPeakState::Thin,
            false,
            0,
            ReplayTouch::default(),
            ReplayTouch::default(),
            ReplayStabilityState::NoReplayField,
            ReplayComputeState::None,
        ));
        assert_eq!(decision.verdict, ReplayVerdict::NoReplayEvidence);
        assert_eq!(decision.action, ReplayAction::UseRawDecision);
        assert!(!decision.safe_to_answer);
    }

    #[test]
    fn replay_core_keeps_stable_focused_field() {
        let decision = evaluate_replay(replay_input(
            RawPeakState::Focused,
            true,
            2,
            ReplayTouch {
                after_net_dot: 256,
                delta_dot: 128,
                field_state: ReplayFieldState::Focused,
            },
            ReplayTouch {
                after_net_dot: 512,
                delta_dot: 384,
                field_state: ReplayFieldState::Focused,
            },
            ReplayStabilityState::StableUnderSoftTouch,
            ReplayComputeState::Ready,
        ));
        assert_eq!(decision.verdict, ReplayVerdict::StableWithReplay);
        assert_eq!(decision.action, ReplayAction::KeepGateDecision);
        assert!(decision.output_pass);
        assert!(!decision.safe_to_answer);
    }

    #[test]
    fn replay_core_rescues_thin_field_under_soft_touch() {
        let decision = evaluate_replay(replay_input(
            RawPeakState::Thin,
            false,
            2,
            ReplayTouch {
                after_net_dot: 192,
                delta_dot: 128,
                field_state: ReplayFieldState::Focused,
            },
            ReplayTouch {
                after_net_dot: 576,
                delta_dot: 512,
                field_state: ReplayFieldState::Focused,
            },
            ReplayStabilityState::StableUnderSoftTouch,
            ReplayComputeState::Ready,
        ));
        assert_eq!(decision.verdict, ReplayVerdict::ReplayRescuedThinField);
        assert_eq!(decision.action, ReplayAction::ReviewReplayRescuedField);
        assert!(!decision.output_pass);
        assert!(!decision.safe_to_answer);
    }

    #[test]
    fn replay_core_stops_destabilized_field() {
        let decision = evaluate_replay(replay_input(
            RawPeakState::Focused,
            true,
            1,
            ReplayTouch {
                after_net_dot: 96,
                delta_dot: -32,
                field_state: ReplayFieldState::Weakened,
            },
            ReplayTouch {
                after_net_dot: 32,
                delta_dot: -96,
                field_state: ReplayFieldState::Weakened,
            },
            ReplayStabilityState::Destabilizing,
            ReplayComputeState::Weak,
        ));
        assert_eq!(decision.verdict, ReplayVerdict::ReplayDestabilizedField);
        assert_eq!(decision.action, ReplayAction::StopRepairOrSplit);
        assert!(decision.output_veto);
    }

    #[test]
    fn replay_core_flags_full_touch_dependency() {
        let decision = evaluate_replay(replay_input(
            RawPeakState::Contested,
            false,
            1,
            ReplayTouch {
                after_net_dot: 96,
                delta_dot: 32,
                field_state: ReplayFieldState::Improved,
            },
            ReplayTouch {
                after_net_dot: 256,
                delta_dot: 192,
                field_state: ReplayFieldState::Focused,
            },
            ReplayStabilityState::FullTouchRequired,
            ReplayComputeState::Ready,
        ));
        assert_eq!(decision.verdict, ReplayVerdict::ReplayTooStrongRequired);
        assert_eq!(decision.action, ReplayAction::ReviewInterventionDependence);
        assert!(!decision.output_pass);
    }
}
