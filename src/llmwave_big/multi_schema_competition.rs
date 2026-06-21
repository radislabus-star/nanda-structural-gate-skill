//! Multi-schema competition for recurrent L2/L3 candidates.

use serde::Serialize;
use std::cmp::Reverse;

use super::coupled_decode_loop;

pub(crate) const MULTI_SCHEMA_COMPETITION_VERSION: &str =
    "llmwave-big-v560-multi-schema-competition";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct SchemaPeak32 {
    pub schema_id: u32,
    pub route_id: u16,
    pub operator_id: u16,
    pub subject_score: i16,
    pub operator_score: i16,
    pub object_score: i16,
    pub route_score: i16,
    pub splice_penalty: i16,
    pub final_score: i16,
    pub margin: i16,
    pub flags: u16,
    pub reserved: u32,
    pub reserved2: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiSchemaCompetitionReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub decode_bridge_state: &'static str,
    pub query_sequence: Vec<&'static str>,
    pub active_schemas: Vec<ActiveSchemaReport>,
    pub peaks: Vec<SchemaPeakReport>,
    pub selected_route: SelectedRouteReport,
    pub route_splice_trap: RouteSpliceTrap,
    pub metrics: MultiSchemaMetrics,
    pub claim_boundary: MultiSchemaClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct ActiveSchemaReport {
    pub schema_id: u32,
    pub route: &'static str,
    pub form: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SchemaPeakReport {
    pub schema_id: u32,
    pub route: &'static str,
    pub form: &'static str,
    pub subject_score: i16,
    pub operator_score: i16,
    pub object_score: i16,
    pub route_score: i16,
    pub splice_penalty: i16,
    pub final_score: i16,
    pub margin: i16,
    pub record: SchemaPeak32,
}

#[derive(Serialize, Clone)]
pub(crate) struct SelectedRouteReport {
    pub schema_id: u32,
    pub route: &'static str,
    pub sequence: Vec<&'static str>,
    pub competing_routes: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct RouteSpliceTrap {
    pub trap: &'static str,
    pub proposed_sequence: Vec<&'static str>,
    pub individually_plausible: bool,
    pub selected_as_whole_route: bool,
    pub rejected: bool,
    pub reason: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiSchemaMetrics {
    pub active_schema_count: usize,
    pub selected_schema_id: u32,
    pub top_margin: i16,
    pub route_splice_reject_rate: f32,
    pub schema_selection_error_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiSchemaClaimBoundary {
    pub multi_schema_competition_implemented: bool,
    pub fixed_peak_records: bool,
    pub uses_coupled_decode_bridge: bool,
    pub real_corpus_trained: bool,
    pub chat_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

#[derive(Clone, Copy)]
struct ActiveSchema {
    schema_id: u32,
    route_id: u16,
    route: &'static str,
    form: &'static str,
    subject: &'static str,
    operator: &'static str,
    object: &'static str,
}

pub(crate) fn build_multi_schema_competition_report() -> MultiSchemaCompetitionReport {
    let decode = coupled_decode_loop::build_coupled_decode_loop_report();
    let schemas = active_schemas();
    let query = ["Honglu", "issues", "invoice"];
    let mut peaks = schemas
        .iter()
        .map(|schema| score_schema(*schema, query))
        .collect::<Vec<_>>();
    peaks.sort_by_key(|peak| Reverse(peak.final_score));
    let top_score = peaks[0].final_score;
    let runner_up_score = peaks[1].final_score;
    let top_margin = top_score.saturating_sub(runner_up_score);
    for peak in &mut peaks {
        peak.margin = peak.final_score.saturating_sub(runner_up_score);
    }
    let selected = peaks[0];
    let splice_trap = route_splice_trap(&schemas);
    let schema_selection_error_rate = if selected.schema_id == 101 { 0.0 } else { 1.0 };
    let route_splice_reject_rate = if splice_trap.rejected { 1.0 } else { 0.0 };
    let state = if selected.schema_id == 101 && splice_trap.rejected && top_margin > 0 {
        "MULTI_SCHEMA_COMPETITION_READY_NOT_CHAT"
    } else {
        "MULTI_SCHEMA_COMPETITION_REVIEW"
    };

    MultiSchemaCompetitionReport {
        mode: "llmwave-big-multi-schema-competition",
        version: MULTI_SCHEMA_COMPETITION_VERSION,
        roadmap_block: "v521-v560",
        verdict: state,
        decode_bridge_state: decode.verdict,
        query_sequence: query.to_vec(),
        active_schemas: schemas
            .iter()
            .map(|schema| ActiveSchemaReport {
                schema_id: schema.schema_id,
                route: schema.route,
                form: schema.form,
            })
            .collect(),
        peaks: peaks
            .iter()
            .map(|peak| SchemaPeakReport {
                schema_id: peak.schema_id,
                route: route_label(peak.schema_id),
                form: form_label(peak.schema_id),
                subject_score: peak.subject_score,
                operator_score: peak.operator_score,
                object_score: peak.object_score,
                route_score: peak.route_score,
                splice_penalty: peak.splice_penalty,
                final_score: peak.final_score,
                margin: peak.margin,
                record: *peak,
            })
            .collect(),
        selected_route: SelectedRouteReport {
            schema_id: selected.schema_id,
            route: route_label(selected.schema_id),
            sequence: query.to_vec(),
            competing_routes: schemas.len(),
        },
        route_splice_trap: splice_trap,
        metrics: MultiSchemaMetrics {
            active_schema_count: schemas.len(),
            selected_schema_id: selected.schema_id,
            top_margin,
            route_splice_reject_rate,
            schema_selection_error_rate,
            state,
        },
        claim_boundary: MultiSchemaClaimBoundary {
            multi_schema_competition_implemented: true,
            fixed_peak_records: core::mem::size_of::<SchemaPeak32>() == 32,
            uses_coupled_decode_bridge: true,
            real_corpus_trained: false,
            chat_ready: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "Multiple active schemas can compete and a route-splice sequence can be rejected in a tiny fixture",
        },
    }
}

fn active_schemas() -> [ActiveSchema; 4] {
    [
        ActiveSchema {
            schema_id: 101,
            route_id: 31,
            route: "supplier-docs",
            form: "supplier issues invoice",
            subject: "Honglu",
            operator: "issues",
            object: "invoice",
        },
        ActiveSchema {
            schema_id: 102,
            route_id: 32,
            route: "buyer-payment",
            form: "buyer pays invoice",
            subject: "Rustrade",
            operator: "pays",
            object: "invoice",
        },
        ActiveSchema {
            schema_id: 103,
            route_id: 33,
            route: "customs-check",
            form: "customs checks declaration",
            subject: "customs",
            operator: "checks",
            object: "declaration",
        },
        ActiveSchema {
            schema_id: 104,
            route_id: 34,
            route: "lab-protocol",
            form: "lab issues protocol",
            subject: "lab",
            operator: "issues",
            object: "protocol",
        },
    ]
}

fn score_schema(schema: ActiveSchema, query: [&'static str; 3]) -> SchemaPeak32 {
    let subject_score = role_score(schema.subject, query[0]);
    let operator_score = role_score(schema.operator, query[1]);
    let object_score = role_score(schema.object, query[2]);
    let route_score = if subject_score > 0 && operator_score > 0 && object_score > 0 {
        38
    } else if operator_score > 0 || object_score > 0 {
        12
    } else {
        -18
    };
    let splice_penalty = if subject_score <= 0 || operator_score <= 0 || object_score <= 0 {
        24
    } else {
        0
    };
    let final_score = subject_score
        .saturating_add(operator_score)
        .saturating_add(object_score)
        .saturating_add(route_score)
        .saturating_sub(splice_penalty);

    SchemaPeak32 {
        schema_id: schema.schema_id,
        route_id: schema.route_id,
        operator_id: operator_id(schema.operator),
        subject_score,
        operator_score,
        object_score,
        route_score,
        splice_penalty,
        final_score,
        margin: 0,
        flags: 1,
        reserved: 0,
        reserved2: 0,
    }
}

fn role_score(expected: &str, actual: &str) -> i16 {
    if expected == actual {
        64
    } else {
        -36
    }
}

fn operator_id(operator: &str) -> u16 {
    match operator {
        "issues" => 3,
        "pays" => 4,
        "checks" => 5,
        _ => 0,
    }
}

fn route_splice_trap(schemas: &[ActiveSchema; 4]) -> RouteSpliceTrap {
    let proposed = ["Honglu", "pays", "invoice"];
    let individually_plausible = schemas.iter().any(|schema| schema.subject == proposed[0])
        && schemas.iter().any(|schema| schema.operator == proposed[1])
        && schemas.iter().any(|schema| schema.object == proposed[2]);
    let selected_as_whole_route = schemas.iter().any(|schema| {
        schema.subject == proposed[0]
            && schema.operator == proposed[1]
            && schema.object == proposed[2]
    });
    RouteSpliceTrap {
        trap: "route_splice_honglu_pays_invoice",
        proposed_sequence: proposed.to_vec(),
        individually_plausible,
        selected_as_whole_route,
        rejected: individually_plausible && !selected_as_whole_route,
        reason: "pieces_exist_in_competing_schemas_but_no_single_schema_owns_the_whole_route",
    }
}

fn route_label(schema_id: u32) -> &'static str {
    match schema_id {
        101 => "supplier-docs",
        102 => "buyer-payment",
        103 => "customs-check",
        104 => "lab-protocol",
        _ => "unknown",
    }
}

fn form_label(schema_id: u32) -> &'static str {
    match schema_id {
        101 => "supplier issues invoice",
        102 => "buyer pays invoice",
        103 => "customs checks declaration",
        104 => "lab issues protocol",
        _ => "unknown",
    }
}
