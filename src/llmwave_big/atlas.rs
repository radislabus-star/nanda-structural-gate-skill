//! Cold and warm Wave Atlas boundary for long-term memory.

use serde::Serialize;

use super::{operators, residuals, schemas, symbols};

const ATLAS_VERSION: &str = "llmwave-big-v170-atlas-loader-contract";

#[derive(Serialize, Clone)]
pub(crate) struct AtlasReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub state: &'static str,
    pub file_layout: Vec<FileEntry>,
    pub record_formats: Vec<RecordFormat>,
    pub symbol_dictionary: symbols::SymbolDictionaryReport,
    pub operator_dictionary: operators::OperatorDictionaryReport,
    pub schema_atlas: schemas::SchemaAtlasReport,
    pub residual_store: residuals::ResidualStoreReport,
    pub evidence_store: EvidenceStoreReport,
    pub cartridges: Vec<CartridgeBank>,
    pub indexes: Vec<AtlasIndex>,
    pub doctor: AtlasDoctorReport,
    pub loader_preview: AtlasLoaderPreview,
    pub active_packet_contract: ActivePacketContract,
}

#[derive(Serialize, Clone)]
pub(crate) struct FileEntry {
    pub path: &'static str,
    pub role: &'static str,
    pub residency: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct RecordFormat {
    pub name: &'static str,
    pub bytes: usize,
    pub fields: Vec<&'static str>,
    pub active_core_visibility: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct EvidenceStoreReport {
    pub state: &'static str,
    pub cold_fields: Vec<&'static str>,
    pub active_core_field: &'static str,
    pub sample_refs: Vec<EvidenceRefRecord>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct EvidenceRefRecord {
    pub evidence_ref: u32,
    pub source_id: u32,
    pub line: u32,
    pub timestamp_sec: u64,
    pub confidence: u8,
    pub status: u8,
    pub flags: u16,
}

#[derive(Serialize, Clone)]
pub(crate) struct CartridgeBank {
    pub name: &'static str,
    pub path: &'static str,
    pub role: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct AtlasIndex {
    pub name: &'static str,
    pub input: &'static str,
    pub output: &'static str,
    pub residency: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct AtlasDoctorReport {
    pub version: &'static str,
    pub verdict: &'static str,
    pub duplicate_symbols: usize,
    pub overloaded_operators: usize,
    pub broad_schemas: usize,
    pub isolated_residuals: usize,
    pub source_conflicts: usize,
    pub route_imbalance: usize,
    pub checks: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct AtlasLoaderPreview {
    pub version: &'static str,
    pub input: &'static str,
    pub output: &'static str,
    pub top_symbols: Vec<u32>,
    pub top_operators: Vec<u16>,
    pub top_schemas: Vec<u32>,
    pub top_residuals: Vec<u32>,
    pub negative_lanes: Vec<u32>,
    pub evidence_refs: Vec<u32>,
    pub l2_projection: &'static str,
    pub l3_projection: &'static str,
    pub fits_active_core_contract: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct ActivePacketContract {
    pub output_from_loader: &'static str,
    pub must_contain: Vec<&'static str>,
    pub must_not_contain: Vec<&'static str>,
    pub hot_core_note: &'static str,
}

pub(crate) fn build_atlas_report() -> AtlasReport {
    let symbol_report = symbols::build_symbol_dictionary_report();
    let operator_report = operators::build_operator_dictionary_report();
    let schema_report = schemas::build_schema_atlas_report();
    let residual_report = residuals::build_residual_store_report();

    AtlasReport {
        mode: "llmwave-big-atlas-contract",
        version: ATLAS_VERSION,
        roadmap_block: "v161-v170",
        state: "ATLAS_CONTRACT_READY_NOT_HOT_RUNTIME",
        file_layout: atlas_file_layout(),
        record_formats: record_formats(),
        symbol_dictionary: symbol_report,
        operator_dictionary: operator_report,
        schema_atlas: schema_report,
        residual_store: residual_report,
        evidence_store: build_evidence_store_report(),
        cartridges: cartridge_banks(),
        indexes: atlas_indexes(),
        doctor: atlas_doctor_report(),
        loader_preview: atlas_loader_preview(),
        active_packet_contract: ActivePacketContract {
            output_from_loader: "ActivePacketPreview",
            must_contain: vec![
                "top_symbol_ids",
                "top_operator_ids",
                "top_schema_ids",
                "top_residual_ids",
                "negative_lane_ids",
                "evidence_refs",
            ],
            must_not_contain: vec!["json", "strings", "evidence_text", "heap_hashmaps"],
            hot_core_note: "loader_output_is_a_contract_preview_until_v171_v180_hot_core",
        },
    }
}

fn atlas_file_layout() -> Vec<FileEntry> {
    vec![
        file("atlas/symbols.bin", "symbol_atom_table", "cold_or_warm"),
        file("atlas/operators.bin", "operator_atom_table", "cold_or_warm"),
        file("atlas/schemas.bin", "schema_record_table", "cold_or_warm"),
        file(
            "atlas/residuals.bin",
            "residual_record_table",
            "cold_or_warm",
        ),
        file("atlas/evidence.bin", "cold_evidence_refs", "cold"),
        file("atlas/cartridges/code_rust.bin", "domain_cartridge", "cold"),
        file(
            "atlas/cartridges/business_docs.bin",
            "domain_cartridge",
            "cold",
        ),
        file("atlas/cartridges/customs.bin", "domain_cartridge", "cold"),
        file(
            "atlas/cartridges/language_ru.bin",
            "language_cartridge",
            "cold",
        ),
        file(
            "atlas/cartridges/language_en.bin",
            "language_cartridge",
            "cold",
        ),
    ]
}

fn file(path: &'static str, role: &'static str, residency: &'static str) -> FileEntry {
    FileEntry {
        path,
        role,
        residency,
    }
}

fn record_formats() -> Vec<RecordFormat> {
    vec![
        RecordFormat {
            name: "SymbolAtom",
            bytes: core::mem::size_of::<symbols::SymbolAtom>(),
            fields: vec![
                "id:u32",
                "kind:u8",
                "lang:u8",
                "flags:u16",
                "wave_seed:u32",
                "alias_root:u32",
            ],
            active_core_visibility: "compact_id_and_wave_seed_only",
        },
        RecordFormat {
            name: "OperatorAtom",
            bytes: core::mem::size_of::<operators::OperatorAtom>(),
            fields: vec![
                "id:u16",
                "arity:u8",
                "polarity_rules:u16",
                "phase:u16",
                "inverse_id:u16",
            ],
            active_core_visibility: "operator_id_phase_polarity",
        },
        RecordFormat {
            name: "SchemaRecord",
            bytes: core::mem::size_of::<schemas::SchemaRecord>(),
            fields: vec![
                "id:u32",
                "operator_id:u16",
                "subject_role:u16",
                "object_role:u16",
                "route_id:u16",
                "centroid_id:u32",
                "confidence:u8",
            ],
            active_core_visibility: "schema_id_operator_roles_route_centroid",
        },
        RecordFormat {
            name: "ResidualRecord",
            bytes: core::mem::size_of::<residuals::ResidualRecord>(),
            fields: vec![
                "schema_id:u32",
                "subject_id:u32",
                "object_id:u32",
                "phase_delta:i16",
                "evidence_ref:u32",
            ],
            active_core_visibility: "residual_ids_phase_delta_evidence_ref",
        },
    ]
}

fn build_evidence_store_report() -> EvidenceStoreReport {
    EvidenceStoreReport {
        state: "COLD_EVIDENCE_ONLY",
        cold_fields: vec![
            "source_file",
            "line",
            "timestamp",
            "confidence",
            "canonical_current_archive",
        ],
        active_core_field: "evidence_ref",
        sample_refs: vec![
            EvidenceRefRecord {
                evidence_ref: 10_001,
                source_id: 501,
                line: 42,
                timestamp_sec: 1_782_000_000,
                confidence: 95,
                status: 1,
                flags: 0,
            },
            EvidenceRefRecord {
                evidence_ref: 10_002,
                source_id: 502,
                line: 77,
                timestamp_sec: 1_782_000_120,
                confidence: 88,
                status: 2,
                flags: 0,
            },
        ],
    }
}

fn cartridge_banks() -> Vec<CartridgeBank> {
    vec![
        cartridge(
            "language_ru",
            "atlas/cartridges/language_ru.bin",
            "russian_surface_language",
        ),
        cartridge(
            "language_en",
            "atlas/cartridges/language_en.bin",
            "english_surface_language",
        ),
        cartridge(
            "code_rust",
            "atlas/cartridges/code_rust.bin",
            "rust_code_relations",
        ),
        cartridge(
            "business_docs",
            "atlas/cartridges/business_docs.bin",
            "documents_roles_routes",
        ),
        cartridge(
            "customs",
            "atlas/cartridges/customs.bin",
            "customs_certification_routes",
        ),
        cartridge(
            "finance",
            "atlas/cartridges/finance.bin",
            "financial_relations",
        ),
        cartridge(
            "personal_project",
            "atlas/cartridges/personal_project.bin",
            "user_project_memory",
        ),
    ]
}

fn cartridge(name: &'static str, path: &'static str, role: &'static str) -> CartridgeBank {
    CartridgeBank { name, path, role }
}

fn atlas_indexes() -> Vec<AtlasIndex> {
    vec![
        index(
            "symbol_to_schemas",
            "symbol_id",
            "schema_ids",
            "cold_or_warm",
        ),
        index(
            "operator_to_schemas",
            "operator_id",
            "schema_ids",
            "cold_or_warm",
        ),
        index("route_to_schemas", "route_id", "schema_ids", "cold_or_warm"),
        index(
            "entity_to_residuals",
            "symbol_id",
            "residual_ids",
            "cold_or_warm",
        ),
        index(
            "query_wave_to_candidate_schemas",
            "query_wave_signature",
            "candidate_schema_ids",
            "warm_loader",
        ),
    ]
}

fn index(
    name: &'static str,
    input: &'static str,
    output: &'static str,
    residency: &'static str,
) -> AtlasIndex {
    AtlasIndex {
        name,
        input,
        output,
        residency,
    }
}

fn atlas_doctor_report() -> AtlasDoctorReport {
    AtlasDoctorReport {
        version: "v169-atlas-doctor",
        verdict: "ATLAS_SAMPLE_OK",
        duplicate_symbols: 0,
        overloaded_operators: 0,
        broad_schemas: 0,
        isolated_residuals: 0,
        source_conflicts: 0,
        route_imbalance: 0,
        checks: vec![
            "duplicate_symbols",
            "overloaded_operators",
            "schema_too_broad",
            "residual_too_isolated",
            "source_conflict",
            "route_imbalance",
        ],
    }
}

fn atlas_loader_preview() -> AtlasLoaderPreview {
    AtlasLoaderPreview {
        version: "v170-atlas-loader",
        input: "query_wave",
        output: "active_packet_preview",
        top_symbols: vec![1, 2, 3, 4, 5],
        top_operators: vec![1, 2, 3, 4],
        top_schemas: vec![101, 102, 103],
        top_residuals: vec![1_001, 1_002, 1_003],
        negative_lanes: vec![9_001, 9_002],
        evidence_refs: vec![10_001, 10_002],
        l2_projection: "surface_symbol_projection_only",
        l3_projection: "schema_operator_role_route_projection_only",
        fits_active_core_contract: true,
    }
}
