use super::{PackedCentroid1024, PackedTriad32, WAVE_DIM};

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
    let mut signed_query_sum: i64 = 0;
    let triad_energy = (WAVE_DIM as i64) * strength * strength;
    for &query_value in &query.values {
        state = mix64(state);
        let query_value = i64::from(query_value);
        if (state & 1) == 0 {
            signed_query_sum -= query_value;
        } else {
            signed_query_sum += query_value;
        }
    }
    let dot = signed_query_sum * strength;
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
