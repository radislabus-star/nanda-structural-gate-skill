//! Fixed-size contracts for the planned NANDA-6M packed hot core.
//!
//! This module intentionally contains no JSON parsing, strings, maps, or heap
//! containers. It is the byte-level contract that the dynamic CLI layer must
//! pack into before a real cache-resident core can run.

#![allow(dead_code)]

pub const VERSION: &str = "nanda-6m-v1-packed-contract";
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_record_sizes_match_contract() {
        assert_eq!(core::mem::size_of::<PackedTriad32>(), TRIAD_BYTES);
        assert_eq!(core::mem::size_of::<PackedCentroid1024>(), CENTROID_BYTES);
        assert_eq!(core::mem::size_of::<PackedLane64>(), LANE_BYTES);
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
}
