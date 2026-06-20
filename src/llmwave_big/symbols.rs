//! Symbol atom and projection boundary for Atlas-visible entities.

use serde::Serialize;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct SymbolAtom {
    pub id: u32,
    pub kind: u8,
    pub lang: u8,
    pub flags: u16,
    pub wave_seed: u32,
    pub alias_root: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct SymbolDictionaryReport {
    pub version: &'static str,
    pub symbol_kinds: Vec<&'static str>,
    pub languages: Vec<&'static str>,
    pub l2_projection: &'static str,
    pub l3_projection: &'static str,
    pub shared_id_rule: &'static str,
    pub samples: Vec<SymbolAtom>,
}

pub(crate) fn build_symbol_dictionary_report() -> SymbolDictionaryReport {
    SymbolDictionaryReport {
        version: "v162-symbol-dictionary",
        symbol_kinds: vec![
            "word", "root", "morpheme", "entity", "document", "role", "source", "route", "time",
            "number",
        ],
        languages: vec!["neutral", "ru", "en", "code"],
        l2_projection: "token_root_morpheme_word_surface_projection",
        l3_projection: "entity_role_route_document_schema_projection",
        shared_id_rule: "l2_and_l3_share_symbol_ids_but_use_different_projections",
        samples: vec![
            SymbolAtom {
                id: 1,
                kind: 0,
                lang: 2,
                flags: 0,
                wave_seed: 0xA11C_E001,
                alias_root: 1,
            },
            SymbolAtom {
                id: 2,
                kind: 3,
                lang: 0,
                flags: 0,
                wave_seed: 0xA11C_E002,
                alias_root: 2,
            },
            SymbolAtom {
                id: 3,
                kind: 5,
                lang: 0,
                flags: 0,
                wave_seed: 0xA11C_E003,
                alias_root: 3,
            },
        ],
    }
}
