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
