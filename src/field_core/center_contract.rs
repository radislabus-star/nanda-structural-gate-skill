use serde::Serialize;

pub(crate) const FIELD_CENTER_CONTRACT_VERSION: &str = "field-center-contract-v1";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct FieldCenterContract {
    pub version: &'static str,
    pub center_kind: &'static str,
    pub primary_center: Option<String>,
    pub center_strength: f64,
    pub center_gap: f64,
    pub foreign_mass: f64,
    pub near_miss_rejected: bool,
    pub ablation_drop: f64,
    pub read_only: bool,
    pub decision_affects_authority: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

pub(crate) struct FieldCenterContractInput {
    pub center_kind: &'static str,
    pub primary_center: Option<String>,
    pub center_strength: f64,
    pub center_gap: f64,
    pub foreign_mass: f64,
    pub near_miss_rejected: bool,
    pub ablation_drop: f64,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

impl FieldCenterContract {
    pub(crate) fn read_only(input: FieldCenterContractInput) -> Self {
        Self {
            version: FIELD_CENTER_CONTRACT_VERSION,
            center_kind: input.center_kind,
            primary_center: input.primary_center,
            center_strength: input.center_strength,
            center_gap: input.center_gap,
            foreign_mass: input.foreign_mass,
            near_miss_rejected: input.near_miss_rejected,
            ablation_drop: input.ablation_drop,
            read_only: true,
            decision_affects_authority: false,
            safe_claim: input.safe_claim,
            blocked_claims: input.blocked_claims,
        }
    }
}
