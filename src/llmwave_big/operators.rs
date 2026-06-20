//! Operator atom and relation-phase boundary for LLMWave-Big.

use serde::Serialize;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct OperatorAtom {
    pub id: u16,
    pub arity: u8,
    pub polarity_rules: u16,
    pub phase: u16,
    pub inverse_id: u16,
}

#[derive(Serialize, Clone)]
pub(crate) struct OperatorRule {
    pub name: &'static str,
    pub id: u16,
    pub directionality: &'static str,
    pub allowed_subject_roles: Vec<&'static str>,
    pub allowed_object_roles: Vec<&'static str>,
    pub anti_rules: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct OperatorDictionaryReport {
    pub version: &'static str,
    pub atoms: Vec<OperatorAtom>,
    pub rules: Vec<OperatorRule>,
}

pub(crate) fn build_operator_dictionary_report() -> OperatorDictionaryReport {
    OperatorDictionaryReport {
        version: "v163-operator-dictionary",
        atoms: vec![
            atom(1, 2, 0x0011, 0x1001, 0),
            atom(2, 2, 0x0012, 0x1002, 0),
            atom(3, 2, 0x0021, 0x1003, 0),
            atom(4, 2, 0x0022, 0x1004, 0),
            atom(5, 2, 0x0031, 0x1005, 0),
            atom(6, 2, 0x0032, 0x1006, 0),
            atom(7, 2, 0x00F1, 0x1F00, 0),
            atom(8, 2, 0x0041, 0x1008, 8),
            atom(9, 2, 0x0042, 0x1009, 0),
            atom(10, 2, 0x0051, 0x1010, 11),
            atom(11, 2, 0x0052, 0x1011, 10),
            atom(12, 2, 0x0061, 0x1012, 0),
            atom(13, 2, 0x0F01, 0x1F13, 0),
        ],
        rules: vec![
            rule(
                "requires",
                1,
                "directed",
                vec!["process", "document"],
                vec!["document", "condition"],
                vec!["missing_required_object"],
            ),
            rule(
                "supports",
                2,
                "directed",
                vec!["evidence", "document"],
                vec!["claim", "route"],
                vec!["weak_source_support"],
            ),
            rule(
                "issues",
                3,
                "directed",
                vec!["issuer", "supplier"],
                vec!["document"],
                vec!["issuer_buyer_swap"],
            ),
            rule(
                "pays",
                4,
                "directed",
                vec!["payer", "buyer"],
                vec!["payee", "supplier"],
                vec!["payer_payee_swap"],
            ),
            rule(
                "owns",
                5,
                "directed",
                vec!["owner"],
                vec!["asset", "entity"],
                vec!["owner_operator_swap"],
            ),
            rule(
                "causes",
                6,
                "directed",
                vec!["cause"],
                vec!["effect"],
                vec!["reverse_causality"],
            ),
            rule(
                "contradicts",
                7,
                "symmetric",
                vec!["claim"],
                vec!["claim"],
                vec!["conflict_unresolved"],
            ),
            rule(
                "same_as",
                8,
                "symmetric",
                vec!["alias"],
                vec!["canonical"],
                vec!["low_confidence_alias"],
            ),
            rule(
                "part_of",
                9,
                "directed",
                vec!["part"],
                vec!["whole"],
                vec!["whole_part_swap"],
            ),
            rule(
                "before",
                10,
                "directed",
                vec!["event"],
                vec!["event"],
                vec!["temporal_swap"],
            ),
            rule(
                "after",
                11,
                "directed",
                vec!["event"],
                vec!["event"],
                vec!["temporal_swap"],
            ),
            rule(
                "source_weaker_than",
                12,
                "directed",
                vec!["source"],
                vec!["source"],
                vec!["source_rank_conflict"],
            ),
            rule(
                "role_swap",
                13,
                "anti",
                vec!["role"],
                vec!["role"],
                vec!["hard_veto_lane"],
            ),
        ],
    }
}

fn atom(id: u16, arity: u8, polarity_rules: u16, phase: u16, inverse_id: u16) -> OperatorAtom {
    OperatorAtom {
        id,
        arity,
        polarity_rules,
        phase,
        inverse_id,
    }
}

fn rule(
    name: &'static str,
    id: u16,
    directionality: &'static str,
    allowed_subject_roles: Vec<&'static str>,
    allowed_object_roles: Vec<&'static str>,
    anti_rules: Vec<&'static str>,
) -> OperatorRule {
    OperatorRule {
        name,
        id,
        directionality,
        allowed_subject_roles,
        allowed_object_roles,
        anti_rules,
    }
}
