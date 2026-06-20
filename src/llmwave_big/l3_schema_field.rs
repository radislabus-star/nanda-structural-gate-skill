//! L3 cognition field boundary: schemas, operators, roles, and routes.

use serde::Serialize;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct L3ToL2Bias {
    pub route_id: u16,
    pub operator_id: u16,
    pub subject_role: u16,
    pub object_role: u16,
    pub style_id: u16,
    pub strength: i16,
}

pub(crate) fn business_invoice_bias() -> L3ToL2Bias {
    L3ToL2Bias {
        route_id: 31,
        operator_id: 3,
        subject_role: 11,
        object_role: 21,
        style_id: 2,
        strength: 24,
    }
}
