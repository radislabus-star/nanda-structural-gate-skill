use serde::{Deserialize, Serialize};

use super::{
    apply_anti_wave, apply_lens_chain, detect_field_peak, summarize_field_coherence,
    FieldCandidatePeak, FieldClaimBoundary, FieldFamily, FieldLensOperation, FieldPeakSummary,
    FieldTriadProjection, FieldVector1024,
};

pub(crate) const FIELD_PASS_VERSION: &str = "unified-field-pass-v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FieldRecordKind {
    StructuralTriad,
    PackedRecord,
    L2Surface,
    L3Schema,
    FeedbackMemory,
    AntiWaveLane,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FieldRecord {
    pub id: String,
    pub kind: FieldRecordKind,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub route: Option<String>,
    pub group: Option<String>,
    pub confidence: u16,
    pub evidence_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FieldAntiWaveLane {
    pub id: String,
    pub target: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub route: Option<String>,
    pub group: Option<String>,
    pub strength: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldPassInput {
    pub family: FieldFamily,
    pub query: FieldRecord,
    pub records: Vec<FieldRecord>,
    pub lenses: Vec<FieldLensOperation>,
    pub anti_waves: Vec<FieldAntiWaveLane>,
    pub claim_boundary: FieldClaimBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldPassReport {
    pub version: &'static str,
    pub family: FieldFamily,
    pub record_count: usize,
    pub lens_count: usize,
    pub anti_wave_count: usize,
    pub query_signature: String,
    pub focused_signature: String,
    pub peak: FieldPeakSummary,
    pub coherence_state: String,
    pub verdict: String,
    pub safe_to_answer: bool,
    pub claim_boundary: FieldClaimBoundary,
}

impl FieldRecord {
    pub(crate) fn vector(&self) -> FieldVector1024 {
        FieldVector1024::project_record(&FieldTriadProjection {
            subject: self.subject.clone(),
            relation: self.relation.clone(),
            object: self.object.clone(),
            route: self.route.clone(),
            group: self.group.clone(),
        })
    }

    pub(crate) fn synthetic(
        id: impl Into<String>,
        kind: FieldRecordKind,
        subject: impl Into<String>,
        relation: impl Into<String>,
        object: impl Into<String>,
        route: Option<String>,
        group: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            subject: subject.into(),
            relation: relation.into(),
            object: object.into(),
            route,
            group,
            confidence: 255,
            evidence_ref: None,
        }
    }
}

impl FieldAntiWaveLane {
    pub(crate) fn vector(&self) -> FieldVector1024 {
        FieldVector1024::project_record(&FieldTriadProjection {
            subject: self.subject.clone(),
            relation: self.relation.clone(),
            object: self.object.clone(),
            route: self.route.clone(),
            group: self.group.clone(),
        })
    }
}

pub(crate) fn run_field_pass(input: &FieldPassInput) -> FieldPassReport {
    let query = input.query.vector();
    let focused = apply_lens_chain(&query, &input.lenses);
    let mut scoring_query = focused.clone();
    for lane in &input.anti_waves {
        let (reduced, _) = apply_anti_wave(&scoring_query, &lane.vector(), lane.strength);
        scoring_query = reduced;
    }

    let candidates = input
        .records
        .iter()
        .map(|record| FieldCandidatePeak {
            target: record
                .route
                .clone()
                .filter(|route| !route.trim().is_empty())
                .unwrap_or_else(|| record.id.clone()),
            vector: record.vector(),
            support_count: usize::from(record.confidence > 0),
            anti_support_count: 0,
        })
        .collect::<Vec<_>>();
    let peak = detect_field_peak(&scoring_query, &candidates);
    let coherence = summarize_field_coherence(
        &scoring_query,
        Some(&focused),
        &peak,
        input.anti_waves.len(),
    );
    let safe_to_answer = peak.safe_to_answer
        && !input.claim_boundary.not_llm_ready
        && !input.claim_boundary.not_nonlinear_memory_proof
        && input.anti_waves.is_empty();
    let verdict = if safe_to_answer {
        "PASS"
    } else if coherence.field_state == "FIELD_CONTESTED" {
        "VETO"
    } else {
        "WATCH"
    };

    FieldPassReport {
        version: FIELD_PASS_VERSION,
        family: input.family.clone(),
        record_count: input.records.len(),
        lens_count: input.lenses.len(),
        anti_wave_count: input.anti_waves.len(),
        query_signature: query.signature_hex(),
        focused_signature: focused.signature_hex(),
        peak,
        coherence_state: coherence.field_state,
        verdict: verdict.to_string(),
        safe_to_answer,
        claim_boundary: input.claim_boundary.clone(),
    }
}
