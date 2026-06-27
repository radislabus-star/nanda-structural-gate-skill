//! Fixed-size contracts for the planned NANDA-6M packed hot core.
//!
//! This module intentionally contains no JSON parsing, strings, maps, or heap
//! containers. It is the byte-level contract that the dynamic CLI layer must
//! pack into before a real cache-resident core can run.

#![allow(dead_code)]

mod replay;
mod wave;

pub use replay::*;
pub use wave::*;

use std::alloc::{alloc_zeroed, dealloc, handle_alloc_error, Layout};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::slice;

pub const VERSION: &str = "nanda-6m-v40-llmwave-pattern-runtime";
pub const FIELD_RECORD_VIEW_VERSION: &str = "packed-field-record-view-v1";
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
pub const PATTERN_BYTES: usize = 32;
pub const PATTERN_ARENA_BYTES: usize = 524_288;
pub const PATTERN_CAPACITY: usize = PATTERN_ARENA_BYTES / PATTERN_BYTES;
pub const QUERY_WAVE_BYTES: usize = WAVE_DIM * core::mem::size_of::<i16>();
pub const PACKED_MIN_FOCUS_SCORE: f64 = 0.01;
pub const PACKED_MIN_FOCUS_MARGIN: f64 = 0.003;
pub const LANE_FOCUSED_NET_DOT: i64 = 128;
pub const LANE_FOCUSED_DELTA_DOT: i64 = 64;

pub const TRIAD_CAPACITY: usize = TRIAD_ARENA_BYTES / TRIAD_BYTES;
pub const ACTIVE_FIELD_RECORDS: usize = TRIAD_CAPACITY;
pub const CENTROID_CAPACITY: usize = CENTROID_ARENA_BYTES / CENTROID_BYTES;
pub const LANE_CAPACITY: usize = LANE_ARENA_BYTES / LANE_BYTES;
pub const SCORE_BUCKET_CAPACITY: usize = CENTROID_CAPACITY;
pub const RUNTIME_FOCUS_TRIAD_CAPACITY: usize = 15_000;
pub const RUNTIME_FOCUS_FIELD_REQUESTS: usize = 64;
pub const RUNTIME_SCORE_ARRAYS: usize = 3;
pub const RUNTIME_OFFSET_ARRAYS: usize = 2;
pub const RUNTIME_CURSOR_ARRAYS: usize = 1;
pub const HOT_WORKSPACE_ALIGNMENT: usize = 64;
pub const ACTIVE65K_PROOF_FIELDS: usize = 2;
pub const ACTIVE65K_TOP_RECORDS: usize = 16;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedFieldRecordView<'a> {
    triad: &'a PackedTriad32,
}

impl<'a> PackedFieldRecordView<'a> {
    pub const fn new(triad: &'a PackedTriad32) -> Self {
        Self { triad }
    }

    pub const fn source(self) -> &'a PackedTriad32 {
        self.triad
    }

    pub const fn subject_id(self) -> u32 {
        self.triad.subject_id
    }

    pub const fn relation_id(self) -> u16 {
        self.triad.relation_id
    }

    pub const fn object_id(self) -> u32 {
        self.triad.object_id
    }

    pub const fn route_id(self) -> u16 {
        self.triad.route_id
    }

    pub const fn group_id(self) -> u16 {
        self.triad.group_id
    }

    pub const fn confidence(self) -> u8 {
        self.triad.confidence
    }

    pub const fn polarity(self) -> u8 {
        self.triad.polarity
    }

    pub const fn lane_hint(self) -> u16 {
        self.triad.lane_hint
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

#[repr(C)]
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

#[repr(u8)]
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

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedTriadSupportScore {
    pub route_id: u16,
    pub group_id: u16,
    pub record_index: u16,
    pub dot: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedFieldRequest {
    pub axis: PackedAxis,
    pub top_id: u16,
    pub key_hash: u32,
}

impl PackedFieldRequest {
    pub const fn route(top_id: u16, key_hash: u32) -> Self {
        Self {
            axis: PackedAxis::Route,
            top_id,
            key_hash,
        }
    }

    pub const fn group(top_id: u16, key_hash: u32) -> Self {
        Self {
            axis: PackedAxis::Group,
            top_id,
            key_hash,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedSupportBatch {
    pub requested: u16,
    pub built: u16,
    pub considered: u16,
    pub support_count: u16,
    pub anti_count: u16,
    pub checksum: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedBucketBuild {
    pub score_count: u16,
    pub route_count: u16,
    pub group_count: u16,
}

pub struct PackedBucketWorkspace<'a> {
    pub scores_out: &'a mut [PackedTriadSupportScore],
    pub route_sorted_out: &'a mut [PackedTriadSupportScore],
    pub route_offsets_out: &'a mut [u16],
    pub group_sorted_out: &'a mut [PackedTriadSupportScore],
    pub group_offsets_out: &'a mut [u16],
    pub cursors_out: &'a mut [u16],
}

pub struct PackedHotWorkspace<'a> {
    pub buckets: PackedBucketWorkspace<'a>,
    pub fields_out: &'a mut [PackedSupportField],
    pub lanes_out: &'a mut [PackedLane64],
}

pub struct PackedHotWorkspaceArena {
    query_wave: AlignedHotBuffer<i16>,
    requests: AlignedHotBuffer<PackedFieldRequest>,
    scores: AlignedHotBuffer<PackedTriadSupportScore>,
    route_sorted: AlignedHotBuffer<PackedTriadSupportScore>,
    group_sorted: AlignedHotBuffer<PackedTriadSupportScore>,
    route_offsets: AlignedHotBuffer<u16>,
    group_offsets: AlignedHotBuffer<u16>,
    cursors: AlignedHotBuffer<u16>,
    fields: AlignedHotBuffer<PackedSupportField>,
    lanes: AlignedHotBuffer<PackedLane64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedHotWorkspaceArenaLayout {
    pub memory_records: usize,
    pub field_requests: usize,
    pub alignment_bytes: usize,
    pub allocated_bytes: usize,
    pub all_cacheline_aligned: bool,
}

impl PackedHotWorkspaceArena {
    pub fn new(memory_records: usize, field_requests: usize) -> Self {
        Self {
            query_wave: AlignedHotBuffer::zeroed(WAVE_DIM),
            requests: AlignedHotBuffer::zeroed(field_requests),
            scores: AlignedHotBuffer::zeroed(memory_records),
            route_sorted: AlignedHotBuffer::zeroed(memory_records),
            group_sorted: AlignedHotBuffer::zeroed(memory_records),
            route_offsets: AlignedHotBuffer::zeroed(SCORE_BUCKET_CAPACITY + 1),
            group_offsets: AlignedHotBuffer::zeroed(SCORE_BUCKET_CAPACITY + 1),
            cursors: AlignedHotBuffer::zeroed(SCORE_BUCKET_CAPACITY),
            fields: AlignedHotBuffer::zeroed(field_requests),
            lanes: AlignedHotBuffer::zeroed(field_requests),
        }
    }

    pub fn query_wave_mut(&mut self) -> &mut [i16] {
        self.query_wave.as_mut_slice()
    }

    pub fn requests_mut(&mut self) -> &mut [PackedFieldRequest] {
        self.requests.as_mut_slice()
    }

    pub fn workspace_with_requests(
        &mut self,
    ) -> (&mut [PackedFieldRequest], PackedHotWorkspace<'_>) {
        let requests = self.requests.as_mut_slice();
        let workspace = PackedHotWorkspace {
            buckets: PackedBucketWorkspace {
                scores_out: self.scores.as_mut_slice(),
                route_sorted_out: self.route_sorted.as_mut_slice(),
                route_offsets_out: self.route_offsets.as_mut_slice(),
                group_sorted_out: self.group_sorted.as_mut_slice(),
                group_offsets_out: self.group_offsets.as_mut_slice(),
                cursors_out: self.cursors.as_mut_slice(),
            },
            fields_out: self.fields.as_mut_slice(),
            lanes_out: self.lanes.as_mut_slice(),
        };
        (requests, workspace)
    }

    pub fn workspace(&mut self) -> PackedHotWorkspace<'_> {
        PackedHotWorkspace {
            buckets: PackedBucketWorkspace {
                scores_out: self.scores.as_mut_slice(),
                route_sorted_out: self.route_sorted.as_mut_slice(),
                route_offsets_out: self.route_offsets.as_mut_slice(),
                group_sorted_out: self.group_sorted.as_mut_slice(),
                group_offsets_out: self.group_offsets.as_mut_slice(),
                cursors_out: self.cursors.as_mut_slice(),
            },
            fields_out: self.fields.as_mut_slice(),
            lanes_out: self.lanes.as_mut_slice(),
        }
    }

    pub fn layout(&self) -> PackedHotWorkspaceArenaLayout {
        PackedHotWorkspaceArenaLayout {
            memory_records: self.scores.len(),
            field_requests: self.fields.len(),
            alignment_bytes: HOT_WORKSPACE_ALIGNMENT,
            allocated_bytes: self.allocated_bytes(),
            all_cacheline_aligned: self.all_cacheline_aligned(),
        }
    }

    pub fn allocated_bytes(&self) -> usize {
        self.query_wave.bytes()
            + self.requests.bytes()
            + self.scores.bytes()
            + self.route_sorted.bytes()
            + self.group_sorted.bytes()
            + self.route_offsets.bytes()
            + self.group_offsets.bytes()
            + self.cursors.bytes()
            + self.fields.bytes()
            + self.lanes.bytes()
    }

    pub fn all_cacheline_aligned(&self) -> bool {
        self.query_wave.is_aligned()
            && self.requests.is_aligned()
            && self.scores.is_aligned()
            && self.route_sorted.is_aligned()
            && self.group_sorted.is_aligned()
            && self.route_offsets.is_aligned()
            && self.group_offsets.is_aligned()
            && self.cursors.is_aligned()
            && self.fields.is_aligned()
            && self.lanes.is_aligned()
    }
}

pub struct PackedActive65kArena {
    query_wave: AlignedHotBuffer<i16>,
    route_accumulators: AlignedHotBuffer<PackedActiveAccumulator>,
    group_accumulators: AlignedHotBuffer<PackedActiveAccumulator>,
    top_records: AlignedHotBuffer<PackedActiveRecordCandidate>,
    proof_fields: AlignedHotBuffer<PackedSupportField>,
    lanes: AlignedHotBuffer<PackedLane64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedActive65kArenaLayout {
    pub active_records: usize,
    pub route_accumulators: usize,
    pub group_accumulators: usize,
    pub top_records: usize,
    pub proof_fields: usize,
    pub alignment_bytes: usize,
    pub allocated_bytes: usize,
    pub all_cacheline_aligned: bool,
}

impl PackedActive65kArena {
    pub fn new() -> Self {
        Self {
            query_wave: AlignedHotBuffer::zeroed(WAVE_DIM),
            route_accumulators: AlignedHotBuffer::zeroed(SCORE_BUCKET_CAPACITY),
            group_accumulators: AlignedHotBuffer::zeroed(SCORE_BUCKET_CAPACITY),
            top_records: AlignedHotBuffer::zeroed(ACTIVE65K_TOP_RECORDS),
            proof_fields: AlignedHotBuffer::zeroed(ACTIVE65K_PROOF_FIELDS),
            lanes: AlignedHotBuffer::zeroed(ACTIVE65K_PROOF_FIELDS),
        }
    }

    pub fn query_wave_mut(&mut self) -> &mut [i16] {
        self.query_wave.as_mut_slice()
    }

    pub fn reset(&mut self) {
        self.query_wave.fill(0);
        self.route_accumulators
            .fill(PackedActiveAccumulator::default());
        self.group_accumulators
            .fill(PackedActiveAccumulator::default());
        self.top_records
            .fill(PackedActiveRecordCandidate::default());
        self.proof_fields.fill(PackedSupportField::default());
        self.lanes.fill(PackedLane64::default());
    }

    pub fn layout(&self) -> PackedActive65kArenaLayout {
        PackedActive65kArenaLayout {
            active_records: ACTIVE_FIELD_RECORDS,
            route_accumulators: self.route_accumulators.len(),
            group_accumulators: self.group_accumulators.len(),
            top_records: self.top_records.len(),
            proof_fields: self.proof_fields.len(),
            alignment_bytes: HOT_WORKSPACE_ALIGNMENT,
            allocated_bytes: self.allocated_bytes(),
            all_cacheline_aligned: self.all_cacheline_aligned(),
        }
    }

    pub fn allocated_bytes(&self) -> usize {
        self.query_wave.bytes()
            + self.route_accumulators.bytes()
            + self.group_accumulators.bytes()
            + self.top_records.bytes()
            + self.proof_fields.bytes()
            + self.lanes.bytes()
    }

    pub fn all_cacheline_aligned(&self) -> bool {
        self.query_wave.is_aligned()
            && self.route_accumulators.is_aligned()
            && self.group_accumulators.is_aligned()
            && self.top_records.is_aligned()
            && self.proof_fields.is_aligned()
            && self.lanes.is_aligned()
    }
}

impl Default for PackedActive65kArena {
    fn default() -> Self {
        Self::new()
    }
}

struct AlignedHotBuffer<T: Copy> {
    ptr: NonNull<T>,
    len: usize,
    align: usize,
    _marker: PhantomData<T>,
}

impl<T: Copy> AlignedHotBuffer<T> {
    fn zeroed(len: usize) -> Self {
        let align = HOT_WORKSPACE_ALIGNMENT.max(core::mem::align_of::<T>());
        if len == 0 {
            return Self {
                ptr: NonNull::dangling(),
                len,
                align,
                _marker: PhantomData,
            };
        }
        let layout = aligned_layout::<T>(len, align);
        // SAFETY: layout has non-zero size and a power-of-two alignment. The
        // buffer is used only for Copy POD-like hot-core records.
        let raw = unsafe { alloc_zeroed(layout) };
        if raw.is_null() {
            handle_alloc_error(layout);
        }
        let ptr = NonNull::new(raw.cast::<T>()).expect("allocator returned non-null pointer");
        debug_assert_eq!((ptr.as_ptr() as usize) % align, 0);
        Self {
            ptr,
            len,
            align,
            _marker: PhantomData,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn bytes(&self) -> usize {
        self.len.saturating_mul(core::mem::size_of::<T>())
    }

    fn is_aligned(&self) -> bool {
        self.len == 0 || (self.ptr.as_ptr() as usize).is_multiple_of(self.align)
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: ptr was allocated for exactly len contiguous T values and the
        // arena gives out one mutable workspace borrow at a time.
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    fn fill(&mut self, value: T) {
        self.as_mut_slice().fill(value);
    }
}

impl<T: Copy> Drop for AlignedHotBuffer<T> {
    fn drop(&mut self) {
        if self.len == 0 {
            return;
        }
        let layout = aligned_layout::<T>(self.len, self.align);
        // SAFETY: ptr/layout pair matches the allocation created in zeroed.
        unsafe {
            dealloc(self.ptr.as_ptr().cast::<u8>(), layout);
        }
    }
}

fn aligned_layout<T>(len: usize, align: usize) -> Layout {
    let size = core::mem::size_of::<T>()
        .checked_mul(len)
        .expect("hot workspace allocation size overflow");
    Layout::from_size_align(size, align).expect("valid hot workspace layout")
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedHotCycle {
    pub score_count: u16,
    pub route_count: u16,
    pub group_count: u16,
    pub fields_built: u16,
    pub lanes_compiled: u16,
    pub lane_sweep: PackedLaneSweep,
    pub checksum: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedActiveAccumulator {
    pub id: u16,
    pub considered: u32,
    pub support_count: u32,
    pub anti_count: u32,
    pub positive_dot: i64,
    pub negative_dot: i64,
    pub net_dot: i64,
    pub top_record: u16,
    pub top_dot: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedActiveRecordCandidate {
    pub record_index: u16,
    pub route_id: u16,
    pub group_id: u16,
    pub dot: i64,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PackedActive65kState {
    Ready = 1,
    PartialActive = 2,
    EmptyMemory = 3,
    EmptyQuery = 4,
    SpillRequired = 5,
    WorkspaceTooSmall = 6,
    #[default]
    Review = 255,
}

impl PackedActive65kState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ACTIVE_65K_READY",
            Self::PartialActive => "ACTIVE_65K_PARTIAL_ACTIVE_RECORDS",
            Self::EmptyMemory => "ACTIVE_65K_EMPTY_MEMORY",
            Self::EmptyQuery => "ACTIVE_65K_EMPTY_QUERY",
            Self::SpillRequired => "ACTIVE_65K_SPILL_REQUIRED",
            Self::WorkspaceTooSmall => "ACTIVE_65K_WORKSPACE_TOO_SMALL",
            Self::Review => "ACTIVE_65K_REVIEW",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedActive65kUsage {
    pub state: PackedActive65kState,
    pub active_records: usize,
    pub required_records: usize,
    pub centroids: usize,
    pub resident_lanes: usize,
    pub active_hot_bytes: usize,
    pub workspace_required_bytes: usize,
    pub workspace_budget_bytes: usize,
    pub fits_l3: bool,
    pub workspace_fits: bool,
    pub full_active_scan: bool,
    pub streaming_discovery: bool,
    pub proof_rescan: bool,
}

impl PackedActive65kUsage {
    pub fn ready(self) -> bool {
        self.state == PackedActive65kState::Ready
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PackedActive65kDiscovery {
    pub records_scanned: u32,
    pub positive_records: u32,
    pub anti_records: u32,
    pub route_peak: PackedAxisPeak,
    pub group_peak: PackedAxisPeak,
    pub top_record: PackedActiveRecordCandidate,
    pub checksum: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedActive65kProof {
    pub records_scanned: u32,
    pub route_summary: PackedSupportSummary,
    pub group_summary: PackedSupportSummary,
    pub fields_built: u16,
    pub lanes_compiled: u16,
    pub lane_sweep: PackedLaneSweep,
    pub checksum: u64,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PackedActive65kAuthorityState {
    Proved = 1,
    ProofRequired = 2,
    ProofRejected = 3,
    #[default]
    Review = 255,
}

impl PackedActive65kAuthorityState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proved => "PROVED",
            Self::ProofRequired => "PROOF_REQUIRED",
            Self::ProofRejected => "PROOF_REJECTED",
            Self::Review => "AUTHORITY_REVIEW",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedActive65kAuthority {
    pub state: PackedActive65kAuthorityState,
    pub safe_to_answer: bool,
    pub proof_required: bool,
    pub proof_rescan_completed: bool,
    pub candidate_without_proof_can_answer: bool,
    pub candidate_without_proof_can_write_memory: bool,
}

impl PackedActive65kAuthority {
    pub const fn proof_required() -> Self {
        Self {
            state: PackedActive65kAuthorityState::ProofRequired,
            safe_to_answer: false,
            proof_required: true,
            proof_rescan_completed: false,
            candidate_without_proof_can_answer: false,
            candidate_without_proof_can_write_memory: false,
        }
    }

    pub const fn from_proof(decision: PackedPeakDecision, proof_completed: bool) -> Self {
        let proved = proof_completed && decision.safe_to_answer;
        Self {
            state: if proved {
                PackedActive65kAuthorityState::Proved
            } else {
                PackedActive65kAuthorityState::ProofRejected
            },
            safe_to_answer: proved,
            proof_required: !proved,
            proof_rescan_completed: proof_completed,
            candidate_without_proof_can_answer: false,
            candidate_without_proof_can_write_memory: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PackedActive65kRun {
    pub usage: PackedActive65kUsage,
    pub discovery: PackedActive65kDiscovery,
    pub proof: PackedActive65kProof,
    pub decision: PackedPeakDecision,
    pub authority: PackedActive65kAuthority,
    pub ran: bool,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PackedRuntimeState {
    Ready = 1,
    FocusRequired = 2,
    SplitRequired = 3,
    SpillRequired = 4,
    EmptyMemory = 5,
    EmptyQuery = 6,
    WorkspaceTooSmall = 7,
    #[default]
    Review = 255,
}

impl PackedRuntimeState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "PACKED_RUNTIME_READY",
            Self::FocusRequired => "FOCUS_REQUIRED",
            Self::SplitRequired => "SPLIT_REQUIRED",
            Self::SpillRequired => "SPILL_REQUIRED",
            Self::EmptyMemory => "EMPTY_MEMORY",
            Self::EmptyQuery => "EMPTY_QUERY",
            Self::WorkspaceTooSmall => "WORKSPACE_TOO_SMALL",
            Self::Review => "PACKED_RUNTIME_REVIEW",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedRuntimeShape {
    pub memory_records: usize,
    pub centroids: usize,
    pub resident_lanes: usize,
    pub field_requests: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedRuntimeUsage {
    pub state: PackedRuntimeState,
    pub shape: PackedRuntimeShape,
    pub active_hot_bytes: usize,
    pub workspace_required_bytes: usize,
    pub workspace_budget_bytes: usize,
    pub focus_triads_capacity: usize,
    pub max_memory_records_for_requests: usize,
    pub fits_l3: bool,
    pub focus_window_fits: bool,
    pub workspace_fits: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedRuntimeRun {
    pub usage: PackedRuntimeUsage,
    pub cycle: PackedHotCycle,
    pub ran: bool,
}

impl PackedRuntimeUsage {
    pub fn ready(self) -> bool {
        self.state == PackedRuntimeState::Ready
    }
}

pub struct PackedHotCore<'a> {
    memory: &'a [PackedTriad32],
    centroids: usize,
    resident_lanes: usize,
    request_capacity: usize,
    workspace: PackedHotWorkspace<'a>,
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

pub const fn packed_runtime_fixed_workspace_bytes() -> usize {
    QUERY_WAVE_BYTES
        + (RUNTIME_OFFSET_ARRAYS * (SCORE_BUCKET_CAPACITY + 1) * core::mem::size_of::<u16>())
        + (RUNTIME_CURSOR_ARRAYS * SCORE_BUCKET_CAPACITY * core::mem::size_of::<u16>())
}

pub const fn packed_runtime_workspace_bytes(memory_records: usize, field_requests: usize) -> usize {
    packed_runtime_fixed_workspace_bytes()
        + (memory_records * RUNTIME_SCORE_ARRAYS * core::mem::size_of::<PackedTriadSupportScore>())
        + (field_requests
            * (core::mem::size_of::<PackedFieldRequest>()
                + core::mem::size_of::<PackedSupportField>()
                + core::mem::size_of::<PackedLane64>()))
}

pub const fn packed_active65k_workspace_bytes() -> usize {
    QUERY_WAVE_BYTES
        + (SCORE_BUCKET_CAPACITY * core::mem::size_of::<PackedActiveAccumulator>() * 2)
        + (ACTIVE65K_TOP_RECORDS * core::mem::size_of::<PackedActiveRecordCandidate>())
        + (ACTIVE65K_PROOF_FIELDS
            * (core::mem::size_of::<PackedSupportField>() + core::mem::size_of::<PackedLane64>()))
}

pub const fn max_runtime_memory_records_for_requests(field_requests: usize) -> usize {
    let per_record = RUNTIME_SCORE_ARRAYS * core::mem::size_of::<PackedTriadSupportScore>();
    let fixed = packed_runtime_workspace_bytes(0, field_requests);
    if per_record == 0 || fixed >= WORKSPACE_BYTES {
        0
    } else {
        (WORKSPACE_BYTES - fixed) / per_record
    }
}

pub fn validate_active65k_runtime(
    active_records: usize,
    centroids: usize,
    resident_lanes: usize,
) -> PackedActive65kUsage {
    let active_hot_bytes = BudgetUsage {
        active_triads: active_records,
        centroids,
        lanes: resident_lanes,
    }
    .estimated_hot_bytes();
    let workspace_required_bytes = packed_active65k_workspace_bytes();
    let fits_l3 = BudgetUsage {
        active_triads: active_records,
        centroids,
        lanes: resident_lanes,
    }
    .fits();
    let workspace_fits = workspace_required_bytes <= WORKSPACE_BYTES;
    let state = if active_records == 0 {
        PackedActive65kState::EmptyMemory
    } else if active_records > ACTIVE_FIELD_RECORDS
        || centroids > CENTROID_CAPACITY
        || resident_lanes > LANE_CAPACITY
        || active_hot_bytes > BUDGET_BYTES
    {
        PackedActive65kState::SpillRequired
    } else if !workspace_fits {
        PackedActive65kState::WorkspaceTooSmall
    } else if active_records != ACTIVE_FIELD_RECORDS {
        PackedActive65kState::PartialActive
    } else {
        PackedActive65kState::Ready
    };
    PackedActive65kUsage {
        state,
        active_records,
        required_records: ACTIVE_FIELD_RECORDS,
        centroids,
        resident_lanes,
        active_hot_bytes,
        workspace_required_bytes,
        workspace_budget_bytes: WORKSPACE_BYTES,
        fits_l3,
        workspace_fits,
        full_active_scan: state == PackedActive65kState::Ready,
        streaming_discovery: state == PackedActive65kState::Ready,
        proof_rescan: state == PackedActive65kState::Ready,
    }
}

pub fn validate_packed_runtime(shape: PackedRuntimeShape) -> PackedRuntimeUsage {
    let active_hot_bytes = BudgetUsage {
        active_triads: shape.memory_records,
        centroids: shape.centroids,
        lanes: shape.resident_lanes,
    }
    .estimated_hot_bytes();
    let workspace_required_bytes =
        packed_runtime_workspace_bytes(shape.memory_records, shape.field_requests);
    let max_memory_records_for_requests =
        max_runtime_memory_records_for_requests(shape.field_requests);
    let fits_l3 = BudgetUsage {
        active_triads: shape.memory_records,
        centroids: shape.centroids,
        lanes: shape.resident_lanes,
    }
    .fits();
    let focus_window_fits = shape.memory_records <= RUNTIME_FOCUS_TRIAD_CAPACITY;
    let workspace_fits = workspace_required_bytes <= WORKSPACE_BYTES;
    let state = if shape.memory_records == 0 {
        PackedRuntimeState::EmptyMemory
    } else if shape.field_requests == 0 {
        PackedRuntimeState::EmptyQuery
    } else if shape.memory_records > TRIAD_CAPACITY
        || shape.resident_lanes > LANE_CAPACITY
        || active_hot_bytes > BUDGET_BYTES
    {
        PackedRuntimeState::SpillRequired
    } else if shape.centroids > CENTROID_CAPACITY {
        PackedRuntimeState::SplitRequired
    } else if !focus_window_fits || !workspace_fits {
        PackedRuntimeState::FocusRequired
    } else {
        PackedRuntimeState::Ready
    };
    PackedRuntimeUsage {
        state,
        shape,
        active_hot_bytes,
        workspace_required_bytes,
        workspace_budget_bytes: WORKSPACE_BYTES,
        focus_triads_capacity: RUNTIME_FOCUS_TRIAD_CAPACITY,
        max_memory_records_for_requests,
        fits_l3,
        focus_window_fits,
        workspace_fits,
    }
}

pub fn validate_packed_hot_workspace(
    workspace: &PackedHotWorkspace<'_>,
    memory_records: usize,
    field_requests: usize,
) -> bool {
    workspace.buckets.scores_out.len() >= memory_records
        && workspace.buckets.route_sorted_out.len() >= memory_records
        && workspace.buckets.group_sorted_out.len() >= memory_records
        && workspace.buckets.route_offsets_out.len() > SCORE_BUCKET_CAPACITY
        && workspace.buckets.group_offsets_out.len() > SCORE_BUCKET_CAPACITY
        && workspace.buckets.cursors_out.len() >= SCORE_BUCKET_CAPACITY
        && workspace.fields_out.len() >= field_requests
        && workspace.lanes_out.len() >= field_requests
}

impl<'a> PackedHotCore<'a> {
    pub fn attach(
        memory: &'a [PackedTriad32],
        centroids: usize,
        resident_lanes: usize,
        request_capacity: usize,
        workspace: PackedHotWorkspace<'a>,
    ) -> Result<Self, PackedRuntimeUsage> {
        let shape = PackedRuntimeShape {
            memory_records: memory.len(),
            centroids,
            resident_lanes,
            field_requests: request_capacity,
        };
        let mut usage = validate_packed_runtime(shape);
        if usage.ready()
            && !validate_packed_hot_workspace(&workspace, memory.len(), request_capacity)
        {
            usage.state = PackedRuntimeState::WorkspaceTooSmall;
            usage.workspace_fits = false;
        }
        if usage.ready() {
            Ok(Self {
                memory,
                centroids,
                resident_lanes,
                request_capacity,
                workspace,
            })
        } else {
            Err(usage)
        }
    }

    pub fn usage_for_requests(&self, field_requests: usize) -> PackedRuntimeUsage {
        validate_packed_runtime(PackedRuntimeShape {
            memory_records: self.memory.len(),
            centroids: self.centroids,
            resident_lanes: self.resident_lanes,
            field_requests,
        })
    }

    pub fn run_query(
        &mut self,
        query: &PackedWave1024,
        requests: &[PackedFieldRequest],
    ) -> PackedRuntimeRun {
        let mut usage = self.usage_for_requests(requests.len());
        if requests.len() > self.request_capacity
            || !validate_packed_hot_workspace(&self.workspace, self.memory.len(), requests.len())
        {
            usage.state = PackedRuntimeState::WorkspaceTooSmall;
            usage.workspace_fits = false;
            return PackedRuntimeRun {
                usage,
                cycle: PackedHotCycle::default(),
                ran: false,
            };
        }
        if !usage.ready() {
            return PackedRuntimeRun {
                usage,
                cycle: PackedHotCycle::default(),
                ran: false,
            };
        }
        let cycle = run_packed_hot_cycle(self.memory, query, requests, &mut self.workspace);
        PackedRuntimeRun {
            usage,
            cycle,
            ran: true,
        }
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
    for (index, triad) in memory.iter().copied().enumerate() {
        if axis.triad_id(triad) != top_id {
            continue;
        }
        considered = considered.saturating_add(1);
        let dot = score_triad_projection_dot(query, &triad);
        if dot > 0 {
            field.positive_dot += dot;
            support_count = support_count.saturating_add(1);
            set_support_mask(&mut field.support_mask_a, &mut field.support_mask_b, index);
        } else if dot < 0 {
            field.negative_dot += dot;
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
    for (index, (triad, out)) in memory
        .iter()
        .copied()
        .take(count)
        .zip(scores_out.iter_mut())
        .enumerate()
    {
        let dot = score_triad_projection_dot(query, &triad);
        *out = PackedTriadSupportScore {
            route_id: triad.route_id,
            group_id: triad.group_id,
            record_index: index.min(usize::from(u16::MAX)) as u16,
            dot,
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

pub fn build_packed_support_score_buckets(
    memory: &[PackedTriad32],
    query: &PackedWave1024,
    workspace: &mut PackedBucketWorkspace<'_>,
) -> PackedBucketBuild {
    let score_count = build_packed_triad_support_scores(memory, query, workspace.scores_out);
    let score_slice = &workspace.scores_out[..score_count];
    let route_count = bucket_packed_triad_support_scores(
        score_slice,
        PackedAxis::Route,
        workspace.route_sorted_out,
        workspace.route_offsets_out,
        workspace.cursors_out,
    );
    let group_count = bucket_packed_triad_support_scores(
        score_slice,
        PackedAxis::Group,
        workspace.group_sorted_out,
        workspace.group_offsets_out,
        workspace.cursors_out,
    );
    PackedBucketBuild {
        score_count: score_count.min(usize::from(u16::MAX)) as u16,
        route_count: route_count.min(usize::from(u16::MAX)) as u16,
        group_count: group_count.min(usize::from(u16::MAX)) as u16,
    }
}

pub fn build_packed_support_fields_from_score_buckets(
    route_sorted: &[PackedTriadSupportScore],
    route_offsets: &[u16],
    group_sorted: &[PackedTriadSupportScore],
    group_offsets: &[u16],
    requests: &[PackedFieldRequest],
    fields_out: &mut [PackedSupportField],
) -> PackedSupportBatch {
    let count = requests.len().min(fields_out.len());
    let mut batch = PackedSupportBatch {
        requested: requests.len().min(usize::from(u16::MAX)) as u16,
        built: count.min(usize::from(u16::MAX)) as u16,
        ..PackedSupportBatch::default()
    };
    for (request, field_out) in requests
        .iter()
        .copied()
        .take(count)
        .zip(fields_out.iter_mut())
    {
        let summary = match request.axis {
            PackedAxis::Route => build_packed_support_field_from_score_bucket(
                route_sorted,
                route_offsets,
                request.axis,
                request.top_id,
                request.key_hash,
            ),
            PackedAxis::Group => build_packed_support_field_from_score_bucket(
                group_sorted,
                group_offsets,
                request.axis,
                request.top_id,
                request.key_hash,
            ),
        };
        *field_out = summary.field;
        batch.considered = batch.considered.saturating_add(summary.considered);
        batch.support_count = batch.support_count.saturating_add(summary.support_count);
        batch.anti_count = batch.anti_count.saturating_add(summary.anti_count);
        batch.checksum = batch
            .checksum
            .wrapping_add(summary.field.positive_dot as u64)
            .wrapping_add(summary.field.negative_dot as u64)
            .wrapping_add(u64::from(summary.considered))
            .wrapping_add(u64::from(summary.support_count))
            .wrapping_add(u64::from(summary.anti_count));
    }
    batch
}

pub fn run_packed_hot_cycle(
    memory: &[PackedTriad32],
    query: &PackedWave1024,
    requests: &[PackedFieldRequest],
    workspace: &mut PackedHotWorkspace<'_>,
) -> PackedHotCycle {
    let buckets = build_packed_support_score_buckets(memory, query, &mut workspace.buckets);
    let route_count =
        usize::from(buckets.route_count).min(workspace.buckets.route_sorted_out.len());
    let group_count =
        usize::from(buckets.group_count).min(workspace.buckets.group_sorted_out.len());
    let batch = build_packed_support_fields_from_score_buckets(
        &workspace.buckets.route_sorted_out[..route_count],
        workspace.buckets.route_offsets_out,
        &workspace.buckets.group_sorted_out[..group_count],
        workspace.buckets.group_offsets_out,
        requests,
        workspace.fields_out,
    );
    let field_count = usize::from(batch.built).min(workspace.fields_out.len());
    let lane_count = field_count.min(workspace.lanes_out.len());
    let sweep = compile_and_apply_aligned_suppress_anti_lane_sweep(
        &workspace.fields_out[..field_count],
        &mut workspace.lanes_out[..lane_count],
    );
    PackedHotCycle {
        score_count: buckets.score_count,
        route_count: buckets.route_count,
        group_count: buckets.group_count,
        fields_built: batch.built,
        lanes_compiled: sweep.lanes.min(usize::from(u16::MAX)) as u16,
        lane_sweep: sweep,
        checksum: batch
            .checksum
            .wrapping_add(sweep.checksum)
            .wrapping_add(sweep.best_after_net_dot as u64)
            .wrapping_add(sweep.total_delta_dot as u64)
            .wrapping_add(sweep.focused as u64),
    }
}

pub fn run_packed_active65k_cycle(
    memory: &[PackedTriad32],
    query: &PackedWave1024,
    arena: &mut PackedActive65kArena,
    centroids: usize,
    resident_lanes: usize,
) -> PackedActive65kRun {
    let mut usage = validate_active65k_runtime(memory.len(), centroids, resident_lanes);
    let layout = arena.layout();
    if layout.allocated_bytes < usage.workspace_required_bytes || !layout.all_cacheline_aligned {
        usage.state = PackedActive65kState::WorkspaceTooSmall;
        usage.workspace_fits = false;
        return PackedActive65kRun {
            usage,
            ..PackedActive65kRun::default()
        };
    }
    if query.energy_i64() <= 0 {
        usage.state = PackedActive65kState::EmptyQuery;
        return PackedActive65kRun {
            usage,
            ..PackedActive65kRun::default()
        };
    }
    if !usage.ready() {
        return PackedActive65kRun {
            usage,
            ..PackedActive65kRun::default()
        };
    }

    arena.reset();
    let discovery = run_active65k_discovery(
        memory,
        query,
        arena.route_accumulators.as_mut_slice(),
        arena.group_accumulators.as_mut_slice(),
        arena.top_records.as_mut_slice(),
    );
    let proof = run_active65k_proof_rescan(
        memory,
        query,
        discovery.route_peak.top_id,
        discovery.group_peak.top_id,
        arena.proof_fields.as_mut_slice(),
        arena.lanes.as_mut_slice(),
    );
    let decision = evaluate_packed_peak_decision(
        discovery.route_peak,
        discovery.group_peak,
        query.energy_i64().max(0) as u64,
        memory.len(),
        1,
    );
    let proof_completed = proof.records_scanned as usize == memory.len()
        && usize::from(proof.fields_built) == ACTIVE65K_PROOF_FIELDS
        && usize::from(proof.lanes_compiled) == ACTIVE65K_PROOF_FIELDS;
    PackedActive65kRun {
        usage,
        discovery,
        proof,
        decision,
        authority: PackedActive65kAuthority::from_proof(decision, proof_completed),
        ran: true,
    }
}

pub fn run_packed_active65k_discovery_cycle(
    memory: &[PackedTriad32],
    query: &PackedWave1024,
    arena: &mut PackedActive65kArena,
    centroids: usize,
    resident_lanes: usize,
) -> PackedActive65kRun {
    let mut usage = validate_active65k_runtime(memory.len(), centroids, resident_lanes);
    usage.proof_rescan = false;
    let layout = arena.layout();
    if layout.allocated_bytes < usage.workspace_required_bytes || !layout.all_cacheline_aligned {
        usage.state = PackedActive65kState::WorkspaceTooSmall;
        usage.workspace_fits = false;
        return PackedActive65kRun {
            usage,
            authority: PackedActive65kAuthority::proof_required(),
            ..PackedActive65kRun::default()
        };
    }
    if query.energy_i64() <= 0 {
        usage.state = PackedActive65kState::EmptyQuery;
        return PackedActive65kRun {
            usage,
            authority: PackedActive65kAuthority::proof_required(),
            ..PackedActive65kRun::default()
        };
    }
    if !usage.ready() {
        return PackedActive65kRun {
            usage,
            authority: PackedActive65kAuthority::proof_required(),
            ..PackedActive65kRun::default()
        };
    }

    arena.reset();
    let discovery = run_active65k_discovery(
        memory,
        query,
        arena.route_accumulators.as_mut_slice(),
        arena.group_accumulators.as_mut_slice(),
        arena.top_records.as_mut_slice(),
    );
    let mut decision = evaluate_packed_peak_decision(
        discovery.route_peak,
        discovery.group_peak,
        query.energy_i64().max(0) as u64,
        memory.len(),
        1,
    );
    decision.verdict_pass = false;
    decision.safe_to_answer = false;
    PackedActive65kRun {
        usage,
        discovery,
        proof: PackedActive65kProof::default(),
        decision,
        authority: PackedActive65kAuthority::proof_required(),
        ran: true,
    }
}

fn run_active65k_discovery(
    memory: &[PackedTriad32],
    query: &PackedWave1024,
    route_accumulators: &mut [PackedActiveAccumulator],
    group_accumulators: &mut [PackedActiveAccumulator],
    top_records: &mut [PackedActiveRecordCandidate],
) -> PackedActive65kDiscovery {
    route_accumulators.fill(PackedActiveAccumulator::default());
    group_accumulators.fill(PackedActiveAccumulator::default());
    top_records.fill(PackedActiveRecordCandidate::default());

    let query_energy = query.energy_i64();
    let mut records_scanned = 0u32;
    let mut positive_records = 0u32;
    let mut anti_records = 0u32;
    let mut top_record = PackedActiveRecordCandidate::default();
    let mut checksum = 0u64;
    for (index, triad) in memory.iter().copied().enumerate() {
        let dot = score_triad_projection_dot(query, &triad);
        let record_index = index.min(usize::from(u16::MAX)) as u16;
        records_scanned = records_scanned.saturating_add(1);
        if dot > 0 {
            positive_records = positive_records.saturating_add(1);
        } else if dot < 0 {
            anti_records = anti_records.saturating_add(1);
        }
        accumulate_active_axis(route_accumulators, triad.route_id, record_index, dot);
        accumulate_active_axis(group_accumulators, triad.group_id, record_index, dot);
        let candidate = PackedActiveRecordCandidate {
            record_index,
            route_id: triad.route_id,
            group_id: triad.group_id,
            dot,
        };
        if candidate.dot > 0
            && (top_record.dot <= 0
                || active_record_order(&candidate, &top_record) == core::cmp::Ordering::Greater)
        {
            top_record = candidate;
        }
        checksum = checksum
            .wrapping_add(dot as u64)
            .wrapping_add(u64::from(triad.route_id) << 7)
            .wrapping_add(u64::from(triad.group_id) << 17)
            .wrapping_add(u64::from(record_index));
    }
    if let Some(slot) = top_records.first_mut() {
        *slot = top_record;
    }
    PackedActive65kDiscovery {
        records_scanned,
        positive_records,
        anti_records,
        route_peak: active_axis_peak(route_accumulators, query_energy),
        group_peak: active_axis_peak(group_accumulators, query_energy),
        top_record,
        checksum,
    }
}

fn run_active65k_proof_rescan(
    memory: &[PackedTriad32],
    query: &PackedWave1024,
    route_id: u16,
    group_id: u16,
    fields_out: &mut [PackedSupportField],
    lanes_out: &mut [PackedLane64],
) -> PackedActive65kProof {
    fields_out.fill(PackedSupportField::default());
    lanes_out.fill(PackedLane64::default());
    if fields_out.len() < ACTIVE65K_PROOF_FIELDS {
        return PackedActive65kProof::default();
    }
    fields_out[0] = PackedSupportField {
        top_id: route_id,
        key_hash: u32::from(route_id) | 0x6500_0000,
        ..PackedSupportField::default()
    };
    fields_out[1] = PackedSupportField {
        top_id: group_id,
        key_hash: u32::from(group_id) | 0x6501_0000,
        ..PackedSupportField::default()
    };

    let mut route_considered = 0u16;
    let mut route_support = 0u16;
    let mut route_anti = 0u16;
    let mut group_considered = 0u16;
    let mut group_support = 0u16;
    let mut group_anti = 0u16;
    let mut records_scanned = 0u32;
    let mut checksum = 0u64;
    for (index, triad) in memory.iter().copied().enumerate() {
        records_scanned = records_scanned.saturating_add(1);
        let mut proof_dot = None;
        if route_id != 0 && triad.route_id == route_id {
            let dot = *proof_dot.get_or_insert_with(|| score_triad_projection_dot(query, &triad));
            accumulate_support_field_with_dot(
                &mut fields_out[0],
                dot,
                index,
                &mut route_considered,
                &mut route_support,
                &mut route_anti,
            );
        }
        if group_id != 0 && triad.group_id == group_id {
            let dot = *proof_dot.get_or_insert_with(|| score_triad_projection_dot(query, &triad));
            accumulate_support_field_with_dot(
                &mut fields_out[1],
                dot,
                index,
                &mut group_considered,
                &mut group_support,
                &mut group_anti,
            );
        }
        checksum = checksum
            .wrapping_add(proof_dot.unwrap_or(0) as u64)
            .wrapping_add(u64::from(triad.route_id) << 11)
            .wrapping_add(u64::from(triad.group_id) << 19);
    }
    let fields_built = ACTIVE65K_PROOF_FIELDS.min(fields_out.len()) as u16;
    let lane_count = usize::from(fields_built).min(lanes_out.len());
    let lane_sweep =
        compile_and_apply_aligned_suppress_anti_lane_sweep(&fields_out[..lane_count], lanes_out);
    PackedActive65kProof {
        records_scanned,
        route_summary: PackedSupportSummary {
            field: fields_out[0],
            considered: route_considered,
            support_count: route_support,
            anti_count: route_anti,
        },
        group_summary: PackedSupportSummary {
            field: fields_out[1],
            considered: group_considered,
            support_count: group_support,
            anti_count: group_anti,
        },
        fields_built,
        lanes_compiled: lane_sweep.lanes.min(usize::from(u16::MAX)) as u16,
        lane_sweep,
        checksum: checksum
            .wrapping_add(fields_out[0].positive_dot as u64)
            .wrapping_add(fields_out[0].negative_dot as u64)
            .wrapping_add(fields_out[1].positive_dot as u64)
            .wrapping_add(fields_out[1].negative_dot as u64)
            .wrapping_add(lane_sweep.checksum),
    }
}

fn accumulate_active_axis(
    accumulators: &mut [PackedActiveAccumulator],
    id: u16,
    record_index: u16,
    dot: i64,
) {
    let Some(accumulator) = accumulators.get_mut(usize::from(id)) else {
        return;
    };
    accumulator.id = id;
    let was_empty = accumulator.considered == 0;
    accumulator.considered = accumulator.considered.saturating_add(1);
    accumulator.net_dot = accumulator.net_dot.saturating_add(dot);
    if dot > 0 {
        accumulator.positive_dot = accumulator.positive_dot.saturating_add(dot);
        accumulator.support_count = accumulator.support_count.saturating_add(1);
    } else if dot < 0 {
        accumulator.negative_dot = accumulator.negative_dot.saturating_add(dot);
        accumulator.anti_count = accumulator.anti_count.saturating_add(1);
    }
    if was_empty || dot > accumulator.top_dot {
        accumulator.top_dot = dot;
        accumulator.top_record = record_index;
    }
}

fn active_axis_peak(accumulators: &[PackedActiveAccumulator], query_energy: i64) -> PackedAxisPeak {
    let mut top_id = 0u16;
    let mut top_score = 0.0f64;
    let mut second_id = 0u16;
    let mut second_score = 0.0f64;
    for accumulator in accumulators.iter().copied() {
        if accumulator.id == 0 || accumulator.considered == 0 || accumulator.net_dot <= 0 {
            continue;
        }
        let score = active_axis_score(accumulator, query_energy);
        if score > top_score {
            second_id = top_id;
            second_score = top_score;
            top_id = accumulator.id;
            top_score = score;
        } else if score > second_score {
            second_id = accumulator.id;
            second_score = score;
        }
    }
    PackedAxisPeak::evaluate(top_id, top_score, second_id, second_score)
}

fn active_axis_score(accumulator: PackedActiveAccumulator, query_energy: i64) -> f64 {
    let denom = (query_energy.max(1) as f64) * (accumulator.considered.max(1) as f64);
    (accumulator.net_dot.max(0) as f64) / denom.max(1.0)
}

fn offer_active_top_record(
    top_records: &mut [PackedActiveRecordCandidate],
    candidate: PackedActiveRecordCandidate,
) {
    if candidate.dot <= 0 || top_records.is_empty() {
        return;
    }
    let mut replace_idx = 0usize;
    let mut replace = top_records[0];
    for (idx, item) in top_records.iter().copied().enumerate().skip(1) {
        if active_record_order(&replace, &item) == core::cmp::Ordering::Greater {
            replace_idx = idx;
            replace = item;
        }
    }
    if replace.dot == 0 || active_record_order(&candidate, &replace) == core::cmp::Ordering::Greater
    {
        top_records[replace_idx] = candidate;
    }
}

fn active_record_order(
    left: &PackedActiveRecordCandidate,
    right: &PackedActiveRecordCandidate,
) -> core::cmp::Ordering {
    left.dot
        .cmp(&right.dot)
        .then_with(|| right.record_index.cmp(&left.record_index))
}

fn accumulate_support_field_with_dot(
    field: &mut PackedSupportField,
    dot: i64,
    index: usize,
    considered: &mut u16,
    support_count: &mut u16,
    anti_count: &mut u16,
) {
    *considered = considered.saturating_add(1);
    if dot > 0 {
        field.positive_dot = field.positive_dot.saturating_add(dot);
        *support_count = support_count.saturating_add(1);
        set_support_mask(&mut field.support_mask_a, &mut field.support_mask_b, index);
    } else if dot < 0 {
        field.negative_dot = field.negative_dot.saturating_add(dot);
        *anti_count = anti_count.saturating_add(1);
        set_support_mask(&mut field.anti_mask_a, &mut field.anti_mask_b, index);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_record_sizes_match_contract() {
        assert_eq!(core::mem::size_of::<PackedTriad32>(), TRIAD_BYTES);
        assert_eq!(
            core::mem::size_of::<PackedFieldRecordView<'static>>(),
            core::mem::size_of::<&PackedTriad32>()
        );
        assert_eq!(core::mem::size_of::<PackedCentroid1024>(), CENTROID_BYTES);
        assert_eq!(core::mem::size_of::<PackedLane64>(), LANE_BYTES);
        assert_eq!(core::mem::size_of::<PackedWave1024>(), QUERY_WAVE_BYTES);
        assert_eq!(core::mem::size_of::<PackedTriadSupportScore>(), 16);
        assert_eq!(core::mem::size_of::<PackedFieldRequest>(), 8);
        assert_eq!(core::mem::size_of::<PackedSupportField>(), 56);
    }

    #[test]
    fn packed_field_record_view_borrows_triad_axes() {
        let triad = PackedTriad32::new(PackedTriadInput {
            subject_id: 10,
            relation_id: 20,
            object_id: 30,
            route_id: 40,
            group_id: 50,
            confidence: 200,
            polarity: 1,
            lane_hint: 60,
            ..PackedTriadInput::default()
        });
        let view = PackedFieldRecordView::new(&triad);

        assert_eq!(view.source(), &triad);
        assert_eq!(view.subject_id(), 10);
        assert_eq!(view.relation_id(), 20);
        assert_eq!(view.object_id(), 30);
        assert_eq!(view.route_id(), 40);
        assert_eq!(view.group_id(), 50);
        assert_eq!(view.confidence(), 200);
        assert_eq!(view.polarity(), 1);
        assert_eq!(view.lane_hint(), 60);
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
    fn runtime_contract_refuses_unfocused_hot_workspace() {
        let focused = validate_packed_runtime(PackedRuntimeShape {
            memory_records: 64,
            centroids: 18,
            resident_lanes: 2,
            field_requests: 64,
        });
        assert_eq!(focused.state, PackedRuntimeState::Ready);
        assert!(focused.ready());
        assert!(focused.focus_window_fits);
        assert!(focused.workspace_fits);
        assert!(focused.max_memory_records_for_requests > 10_000);

        let focused_15k = validate_packed_runtime(PackedRuntimeShape {
            memory_records: RUNTIME_FOCUS_TRIAD_CAPACITY,
            centroids: 18,
            resident_lanes: 2,
            field_requests: RUNTIME_FOCUS_FIELD_REQUESTS,
        });
        assert_eq!(focused_15k.state, PackedRuntimeState::Ready);
        assert!(focused_15k.focus_window_fits);
        assert!(focused_15k.workspace_fits);

        let just_over_focus = validate_packed_runtime(PackedRuntimeShape {
            memory_records: RUNTIME_FOCUS_TRIAD_CAPACITY + 1,
            centroids: 18,
            resident_lanes: 2,
            field_requests: RUNTIME_FOCUS_FIELD_REQUESTS,
        });
        assert_eq!(just_over_focus.state, PackedRuntimeState::FocusRequired);
        assert!(!just_over_focus.focus_window_fits);
        assert!(just_over_focus.workspace_fits);

        let unfocused = validate_packed_runtime(PackedRuntimeShape {
            memory_records: TRIAD_CAPACITY,
            centroids: 2,
            resident_lanes: 0,
            field_requests: 64,
        });
        assert_eq!(unfocused.state, PackedRuntimeState::FocusRequired);
        assert!(!unfocused.ready());
        assert!(!unfocused.focus_window_fits);
        assert!(!unfocused.workspace_fits);
        assert!(unfocused.workspace_required_bytes > WORKSPACE_BYTES);
    }

    #[test]
    fn packed_hot_core_refuses_short_workspace() {
        let memory = [PackedTriad32::new(PackedTriadInput {
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
        })];
        let mut scores: [PackedTriadSupportScore; 0] = [];
        let mut route_sorted = [PackedTriadSupportScore::default(); 1];
        let mut group_sorted = [PackedTriadSupportScore::default(); 1];
        let mut route_offsets = [0u16; SCORE_BUCKET_CAPACITY + 1];
        let mut group_offsets = [0u16; SCORE_BUCKET_CAPACITY + 1];
        let mut cursors = [0u16; SCORE_BUCKET_CAPACITY];
        let mut fields = [PackedSupportField::default(); 1];
        let mut lanes = [PackedLane64::default(); 1];
        let workspace = PackedHotWorkspace {
            buckets: PackedBucketWorkspace {
                scores_out: &mut scores,
                route_sorted_out: &mut route_sorted,
                route_offsets_out: &mut route_offsets,
                group_sorted_out: &mut group_sorted,
                group_offsets_out: &mut group_offsets,
                cursors_out: &mut cursors,
            },
            fields_out: &mut fields,
            lanes_out: &mut lanes,
        };
        let err = match PackedHotCore::attach(&memory, 2, 0, 1, workspace) {
            Ok(_) => panic!("short workspace must be refused"),
            Err(err) => err,
        };
        assert_eq!(err.state, PackedRuntimeState::WorkspaceTooSmall);
    }

    #[test]
    fn packed_hot_workspace_arena_is_aligned_and_reusable() {
        let memory = [
            PackedTriad32::new(PackedTriadInput {
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
            }),
            PackedTriad32::new(PackedTriadInput {
                subject_id: 2,
                object_id: 3,
                evidence_ref: 4,
                wave_seed: 5,
                relation_id: 6,
                route_id: 7,
                group_id: 11,
                role_pack: 0x0201,
                flags: 1,
                lane_hint: 0,
                check: 9,
                confidence: 220,
                polarity: 0,
            }),
        ];
        let query = project_triads(&memory);
        let requests = [
            PackedFieldRequest::route(7, 101),
            PackedFieldRequest::group(9, 102),
        ];
        let mut arena = PackedHotWorkspaceArena::new(memory.len(), requests.len());
        let layout = arena.layout();
        assert_eq!(layout.memory_records, memory.len());
        assert_eq!(layout.field_requests, requests.len());
        assert_eq!(layout.alignment_bytes, HOT_WORKSPACE_ALIGNMENT);
        assert!(layout.all_cacheline_aligned);
        assert_eq!(
            layout.allocated_bytes,
            packed_runtime_workspace_bytes(memory.len(), requests.len())
        );

        let (request_workspace, workspace) = arena.workspace_with_requests();
        request_workspace.copy_from_slice(&requests);
        assert!(validate_packed_hot_workspace(
            &workspace,
            memory.len(),
            requests.len()
        ));
        let mut core = PackedHotCore::attach(&memory, 4, 0, requests.len(), workspace)
            .expect("aligned arena should satisfy hot workspace contract");
        let first = core.run_query(&query, request_workspace);
        let second = core.run_query(&query, request_workspace);
        assert!(first.ran);
        assert!(second.ran);
        assert_eq!(first.cycle.score_count, memory.len() as u16);
        assert_eq!(second.cycle.score_count, memory.len() as u16);
        assert_eq!(first.cycle.checksum, second.cycle.checksum);
    }

    #[test]
    fn active65k_streaming_cycle_scans_full_field_without_score_arrays() {
        let memory: Vec<PackedTriad32> = (0..ACTIVE_FIELD_RECORDS)
            .map(|idx| {
                let idx = idx as u32;
                PackedTriad32::new(PackedTriadInput {
                    subject_id: 1_000 + idx,
                    object_id: 10_000 + idx.wrapping_mul(3),
                    evidence_ref: 100_000 + idx.wrapping_mul(7),
                    wave_seed: 0x6500_0000u32.wrapping_add(idx.wrapping_mul(97)),
                    relation_id: (10 + (idx % 31)) as u16,
                    route_id: (1 + (idx % 7)) as u16,
                    group_id: (1 + (idx % 11)) as u16,
                    role_pack: (((idx % 16) as u16) << 8) | ((idx % 13) as u16),
                    flags: (idx % 5) as u16,
                    lane_hint: (idx % 17) as u16,
                    check: 0x55aa ^ (idx as u16).wrapping_mul(13),
                    confidence: 160 + (idx % 80) as u8,
                    polarity: (idx % 2) as u8,
                })
            })
            .collect();
        let query = project_triads(&memory[..8]);
        let mut arena = PackedActive65kArena::new();
        let layout = arena.layout();
        assert_eq!(layout.active_records, ACTIVE_FIELD_RECORDS);
        assert_eq!(layout.allocated_bytes, packed_active65k_workspace_bytes());
        assert!(layout.allocated_bytes <= WORKSPACE_BYTES);
        assert!(layout.all_cacheline_aligned);

        let run = run_packed_active65k_cycle(&memory, &query, &mut arena, 18, 0);

        assert!(run.ran);
        assert_eq!(run.usage.state, PackedActive65kState::Ready);
        assert_eq!(run.discovery.records_scanned as usize, ACTIVE_FIELD_RECORDS);
        assert_eq!(run.proof.records_scanned as usize, ACTIVE_FIELD_RECORDS);
        assert_eq!(run.proof.fields_built as usize, ACTIVE65K_PROOF_FIELDS);
        assert_eq!(run.proof.lanes_compiled as usize, ACTIVE65K_PROOF_FIELDS);
        assert!(run.usage.full_active_scan);
        assert!(run.usage.streaming_discovery);
        assert!(run.usage.proof_rescan);
        assert!(run.discovery.route_peak.top_id > 0);
        assert!(run.discovery.group_peak.top_id > 0);
        assert!(run.discovery.checksum > 0);
        assert!(run.proof.checksum > 0);
        assert!(run.authority.proof_rescan_completed);
        assert!(!run.authority.candidate_without_proof_can_answer);
        assert!(!run.authority.candidate_without_proof_can_write_memory);
    }

    #[test]
    fn active65k_discovery_cycle_requires_proof_before_authority() {
        let memory: Vec<PackedTriad32> = (0..ACTIVE_FIELD_RECORDS)
            .map(|idx| {
                let idx = idx as u32;
                PackedTriad32::new(PackedTriadInput {
                    subject_id: 1_000 + idx,
                    object_id: 10_000 + idx.wrapping_mul(3),
                    evidence_ref: 100_000 + idx.wrapping_mul(7),
                    wave_seed: 0x6500_0000u32.wrapping_add(idx.wrapping_mul(97)),
                    relation_id: (10 + (idx % 31)) as u16,
                    route_id: (1 + (idx % 7)) as u16,
                    group_id: (1 + (idx % 11)) as u16,
                    role_pack: (((idx % 16) as u16) << 8) | ((idx % 13) as u16),
                    flags: (idx % 5) as u16,
                    lane_hint: (idx % 17) as u16,
                    check: 0x55aa ^ (idx as u16).wrapping_mul(13),
                    confidence: 160 + (idx % 80) as u8,
                    polarity: (idx % 2) as u8,
                })
            })
            .collect();
        let query = project_triads(&memory[..8]);
        let mut arena = PackedActive65kArena::new();

        let run = run_packed_active65k_discovery_cycle(&memory, &query, &mut arena, 18, 0);

        assert!(run.ran);
        assert_eq!(run.usage.state, PackedActive65kState::Ready);
        assert_eq!(run.discovery.records_scanned as usize, ACTIVE_FIELD_RECORDS);
        assert_eq!(run.proof.records_scanned, 0);
        assert!(!run.usage.proof_rescan);
        assert_eq!(
            run.authority.state,
            PackedActive65kAuthorityState::ProofRequired
        );
        assert!(run.authority.proof_required);
        assert!(!run.authority.safe_to_answer);
        assert!(!run.authority.proof_rescan_completed);
        assert!(!run.authority.candidate_without_proof_can_answer);
        assert!(!run.authority.candidate_without_proof_can_write_memory);
        assert!(!run.decision.safe_to_answer);
        assert!(!run.decision.verdict_pass);
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
    fn packed_hot_cycle_builds_fields_and_lanes() {
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
            subject_id: 10,
            object_id: 20,
            evidence_ref: 30,
            wave_seed: 40,
            relation_id: 5,
            route_id: 11,
            group_id: 12,
            role_pack: 0x0201,
            flags: 1,
            lane_hint: 0,
            check: 8,
            confidence: 230,
            polarity: 0,
        });
        let memory = [support, anti, foreign];
        let query = project_triads(&[support]);
        let requests = [
            PackedFieldRequest::route(7, 0x55),
            PackedFieldRequest::group(9, 0x66),
        ];
        let mut scores = [PackedTriadSupportScore::default(); 3];
        let mut route_sorted = [PackedTriadSupportScore::default(); 3];
        let mut group_sorted = [PackedTriadSupportScore::default(); 3];
        let mut route_offsets = [0u16; SCORE_BUCKET_CAPACITY + 1];
        let mut group_offsets = [0u16; SCORE_BUCKET_CAPACITY + 1];
        let mut cursors = [0u16; SCORE_BUCKET_CAPACITY];
        let mut fields = [PackedSupportField::default(); 2];
        let mut lanes = [PackedLane64::default(); 2];
        let run = {
            let workspace = PackedHotWorkspace {
                buckets: PackedBucketWorkspace {
                    scores_out: &mut scores,
                    route_sorted_out: &mut route_sorted,
                    route_offsets_out: &mut route_offsets,
                    group_sorted_out: &mut group_sorted,
                    group_offsets_out: &mut group_offsets,
                    cursors_out: &mut cursors,
                },
                fields_out: &mut fields,
                lanes_out: &mut lanes,
            };
            let mut core = match PackedHotCore::attach(&memory, 4, 0, requests.len(), workspace) {
                Ok(core) => core,
                Err(err) => panic!("focused hot core attach failed: {err:?}"),
            };
            core.run_query(&query, &requests)
        };
        let cycle = run.cycle;

        assert!(run.ran);
        assert_eq!(run.usage.state, PackedRuntimeState::Ready);
        assert_eq!(cycle.score_count, 3);
        assert_eq!(cycle.fields_built, 2);
        assert_eq!(cycle.lanes_compiled, 2);
        assert_eq!(fields[0].top_id, 7);
        assert_eq!(fields[1].top_id, 9);
        assert!(cycle.lane_sweep.applied > 0);
        assert!(cycle.checksum > 0);
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
