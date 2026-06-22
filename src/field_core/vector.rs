use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const FIELD_DIM: usize = 1024;
pub(crate) const FIELD_COMPUTE_VERSION: &str = "unified-field-compute-v1";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct FieldQuerySummary {
    pub source: String,
    pub text: Option<String>,
    pub requested_axes: Vec<String>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldVector1024 {
    values: [i32; FIELD_DIM],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct FieldVectorStats {
    pub dim: usize,
    pub energy: f64,
    pub non_zero: usize,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FieldTriadProjection {
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub route: Option<String>,
    pub group: Option<String>,
}

impl FieldVector1024 {
    pub(crate) fn zero() -> Self {
        Self {
            values: [0; FIELD_DIM],
        }
    }

    pub(crate) fn from_label(label: &str) -> Self {
        let mut values = [0; FIELD_DIM];
        for (idx, slot) in values.iter_mut().enumerate() {
            let mut hasher = Sha256::new();
            hasher.update(label.as_bytes());
            hasher.update((idx as u32).to_le_bytes());
            let digest = hasher.finalize();
            *slot = if digest[0] & 1 == 0 { 1 } else { -1 };
        }
        Self { values }
    }

    pub(crate) fn project_triad(subject: &str, relation: &str, object: &str) -> Self {
        let subject_vector = Self::from_label(&format!("subject:{subject}"));
        let relation_vector = Self::from_label(&format!("relation:{relation}"));
        let object_vector = Self::from_label(&format!("object:{object}"));
        subject_vector.bind(&relation_vector).bind(&object_vector)
    }

    pub(crate) fn project_record(record: &FieldTriadProjection) -> Self {
        let mut vector = Self::project_triad(&record.subject, &record.relation, &record.object);
        if let Some(route) = &record.route {
            vector = vector.bind(&Self::from_label(&format!("route:{route}")));
        }
        if let Some(group) = &record.group {
            vector = vector.bind(&Self::from_label(&format!("group:{group}")));
        }
        vector
    }

    pub(crate) fn bind(&self, other: &Self) -> Self {
        let mut values = [0; FIELD_DIM];
        for ((slot, left), right) in values.iter_mut().zip(&self.values).zip(&other.values) {
            *slot = left.signum() * right.signum();
        }
        Self { values }
    }

    pub(crate) fn bundle(&mut self, other: &Self) {
        for (left, right) in self.values.iter_mut().zip(&other.values) {
            *left = left.saturating_add(*right);
        }
    }

    pub(crate) fn bundle_scaled(&mut self, other: &Self, weight: i32) {
        for (left, right) in self.values.iter_mut().zip(&other.values) {
            *left = left.saturating_add(right.saturating_mul(weight));
        }
    }

    pub(crate) fn subtract_scaled(&mut self, other: &Self, weight: i32) {
        for (left, right) in self.values.iter_mut().zip(&other.values) {
            *left = left.saturating_sub(right.saturating_mul(weight));
        }
    }

    pub(crate) fn dot(&self, other: &Self) -> i64 {
        self.values
            .iter()
            .zip(&other.values)
            .map(|(left, right)| i64::from(*left) * i64::from(*right))
            .sum()
    }

    pub(crate) fn energy(&self) -> f64 {
        self.values
            .iter()
            .map(|value| {
                let value = f64::from(*value);
                value * value
            })
            .sum::<f64>()
            .sqrt()
    }

    pub(crate) fn cosine(&self, other: &Self) -> f64 {
        let energy = self.energy() * other.energy();
        if energy <= f64::EPSILON {
            0.0
        } else {
            self.dot(other) as f64 / energy
        }
    }

    pub(crate) fn signature_hex(&self) -> String {
        let mut hasher = Sha256::new();
        for value in &self.values {
            hasher.update(value.to_le_bytes());
        }
        let digest = hasher.finalize();
        digest
            .iter()
            .take(8)
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    pub(crate) fn stats(&self) -> FieldVectorStats {
        FieldVectorStats {
            dim: FIELD_DIM,
            energy: self.energy(),
            non_zero: self.values.iter().filter(|value| **value != 0).count(),
            signature: self.signature_hex(),
        }
    }
}
