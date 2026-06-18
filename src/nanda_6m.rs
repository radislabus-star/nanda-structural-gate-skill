//! Fixed-size contracts for the planned NANDA-6M packed hot core.
//!
//! This module intentionally contains no JSON parsing, strings, maps, or heap
//! containers. It is the byte-level contract that the dynamic CLI layer must
//! pack into before a real cache-resident core can run.

#![allow(dead_code)]

pub const VERSION: &str = "nanda-6m-v4-hot-replay-core";
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

pub const TRIAD_CAPACITY: usize = TRIAD_ARENA_BYTES / TRIAD_BYTES;
pub const CENTROID_CAPACITY: usize = CENTROID_ARENA_BYTES / CENTROID_BYTES;
pub const LANE_CAPACITY: usize = LANE_ARENA_BYTES / LANE_BYTES;

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
