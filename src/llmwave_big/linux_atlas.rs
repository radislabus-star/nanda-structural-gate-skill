//! Linux Atlas builder for LLMWave-Big.
//!
//! This is cold corpus code. It reads local Linux metadata and writes an
//! append-only fact pack. It deliberately avoids secret-bearing config files and
//! keeps generated data outside the hot loop.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const LINUX_ATLAS_VERSION: &str = "llmwave-big-v-next-linux-atlas";

#[derive(Clone)]
pub(crate) struct LinuxAtlasBuildConfig {
    pub root: PathBuf,
    pub out_dir: PathBuf,
    pub pack_kind: LinuxAtlasPackKind,
    pub max_facts: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LinuxAtlasPackKind {
    Base,
    Delta,
}

impl LinuxAtlasPackKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Delta => "delta",
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxAtlasBuildReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub root: String,
    pub pack_kind: &'static str,
    pub artifact: LinuxAtlasArtifact,
    pub source_stats: LinuxAtlasSourceStats,
    pub route_counts: BTreeMap<String, usize>,
    pub layer_counts: BTreeMap<String, usize>,
    pub outputs: LinuxAtlasOutputs,
    pub sample_facts: Vec<LinuxAtlasFact>,
    pub claim_boundary: LinuxAtlasClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxAtlasArtifact {
    pub atlas_kind: &'static str,
    pub corpus_hash: String,
    pub fact_count: usize,
    pub deduped_facts: usize,
    pub previous_fact_count: usize,
    pub delta_new_facts: usize,
    pub route_count: usize,
    pub layer_count: usize,
    pub append_only: bool,
    pub delta_ready: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct LinuxAtlasSourceStats {
    pub packages_seen: usize,
    pub package_status_facts: usize,
    pub package_file_facts: usize,
    pub apt_packages_seen: usize,
    pub apt_index_facts: usize,
    pub command_index_facts: usize,
    pub translation_facts: usize,
    pub manpage_facts: usize,
    pub systemd_unit_files: usize,
    pub systemd_facts: usize,
    pub runtime_snapshot_facts: usize,
    pub negative_boundary_facts: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxAtlasOutputs {
    pub out_dir: String,
    pub facts_path: String,
    pub manifest_path: String,
    pub current_path: String,
    pub route_index_path: String,
    pub fact_id_index_path: String,
    pub pack_log_path: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxAtlasClaimBoundary {
    pub linux_atlas_built: bool,
    pub append_only_memory_ready: bool,
    pub active_65k_pack_ready: bool,
    pub exposure_layer_ready: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LinuxAtlasFact {
    pub fact_id: String,
    pub layer: String,
    pub domain: String,
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: String,
    pub confidence: u8,
    pub evidence: LinuxAtlasEvidence,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LinuxAtlasEvidence {
    pub source_kind: String,
    pub path: String,
    pub line: usize,
    pub extractor: String,
}

struct FactSink {
    facts: Vec<LinuxAtlasFact>,
    keys: BTreeSet<String>,
    previous_fact_ids: BTreeSet<String>,
    max_facts: usize,
    hash: Sha256,
    stats: LinuxAtlasSourceStats,
}

impl FactSink {
    fn new(max_facts: usize, previous_fact_ids: BTreeSet<String>) -> Self {
        Self {
            facts: Vec::new(),
            keys: BTreeSet::new(),
            previous_fact_ids,
            max_facts,
            hash: Sha256::new(),
            stats: LinuxAtlasSourceStats::default(),
        }
    }

    fn is_full(&self) -> bool {
        self.facts.len() >= self.max_facts
    }

    fn add(&mut self, mut fact: LinuxAtlasFact) {
        normalize_fact(&mut fact);
        let key = fact_key(&fact);
        let new_fact_id = fact_id(&fact.route, &key);
        if self.previous_fact_ids.contains(&new_fact_id) || self.is_full() {
            return;
        }
        if !self.keys.insert(key.clone()) {
            return;
        }
        fact.fact_id = new_fact_id;
        self.hash.update(fact.fact_id.as_bytes());
        self.hash.update(fact.route.as_bytes());
        self.hash.update(fact.subject.as_bytes());
        self.hash.update(fact.relation.as_bytes());
        self.hash.update(fact.object.as_bytes());
        self.facts.push(fact);
    }
}

pub(crate) fn build_linux_atlas_report(
    config: LinuxAtlasBuildConfig,
) -> Result<LinuxAtlasBuildReport> {
    let root = config
        .root
        .canonicalize()
        .with_context(|| format!("canonicalize Linux root {}", config.root.display()))?;
    let previous_fact_ids = if config.pack_kind == LinuxAtlasPackKind::Delta {
        load_existing_fact_ids(&config.out_dir)?
    } else {
        BTreeSet::new()
    };
    let previous_fact_count = previous_fact_ids.len();
    let mut sink = FactSink::new(config.max_facts.max(1), previous_fact_ids);
    collect_os_release(&root, &mut sink)?;
    collect_dpkg_status(&root, &mut sink)?;
    collect_dpkg_file_lists(&root, &mut sink)?;
    collect_manpage_index(&root, &mut sink)?;
    collect_systemd_units(&root, &mut sink)?;
    collect_runtime_snapshot(&root, &mut sink)?;
    add_negative_boundary_facts(&mut sink);
    collect_apt_package_lists(&root, &mut sink)?;
    collect_apt_command_indexes(&root, &mut sink)?;
    collect_apt_translations(&root, &mut sink)?;

    let mut route_counts = BTreeMap::new();
    let mut layer_counts = BTreeMap::new();
    for fact in &sink.facts {
        *route_counts.entry(fact.route.clone()).or_insert(0) += 1;
        *layer_counts.entry(fact.layer.clone()).or_insert(0) += 1;
    }

    let corpus_hash = format!("{:x}", sink.hash.finalize());
    let timestamp = unix_timestamp();
    let facts_path = config.out_dir.join("facts").join(format!(
        "{}-{}.jsonl",
        config.pack_kind.as_str(),
        timestamp
    ));
    let manifest_path = config.out_dir.join("manifest.json");
    let current_path = config
        .out_dir
        .join("consolidated")
        .join("linux-atlas-current.json");
    let route_index_path = config.out_dir.join("indexes").join("route-counts.json");
    let fact_id_index_path = config.out_dir.join("indexes").join("fact-ids.txt");
    let pack_log_path = config.out_dir.join("packs.jsonl");
    let mut known_fact_ids = sink.previous_fact_ids.clone();
    known_fact_ids.extend(sink.facts.iter().map(|fact| fact.fact_id.clone()));
    write_outputs(OutputWriteRequest {
        facts_path: &facts_path,
        manifest_path: &manifest_path,
        current_path: &current_path,
        route_index_path: &route_index_path,
        fact_id_index_path: &fact_id_index_path,
        pack_log_path: &pack_log_path,
        facts: &sink.facts,
        corpus_hash: &corpus_hash,
        route_counts: &route_counts,
        layer_counts: &layer_counts,
        known_fact_ids: &known_fact_ids,
        pack_kind: config.pack_kind,
        stats: &sink.stats,
    })?;

    let fact_count = sink.facts.len();
    let verdict = if fact_count >= 1_000 && config.pack_kind == LinuxAtlasPackKind::Base {
        "LINUX_ATLAS_BASE_READY"
    } else if fact_count >= 1_000 && config.pack_kind == LinuxAtlasPackKind::Delta {
        "LINUX_ATLAS_DELTA_READY"
    } else if fact_count == 0 && config.pack_kind == LinuxAtlasPackKind::Delta {
        "LINUX_ATLAS_DELTA_NO_NEW_FACTS"
    } else if fact_count > 0 {
        "LINUX_ATLAS_SMALL_READY"
    } else {
        "LINUX_ATLAS_EMPTY"
    };

    Ok(LinuxAtlasBuildReport {
        mode: "llmwave-big-linux-atlas-build",
        version: LINUX_ATLAS_VERSION,
        verdict,
        root: root.display().to_string(),
        pack_kind: config.pack_kind.as_str(),
        artifact: LinuxAtlasArtifact {
            atlas_kind: "linux-causal-knowledge-atlas",
            corpus_hash,
            fact_count,
            deduped_facts: sink.keys.len(),
            previous_fact_count,
            delta_new_facts: if config.pack_kind == LinuxAtlasPackKind::Delta {
                fact_count
            } else {
                0
            },
            route_count: route_counts.len(),
            layer_count: layer_counts.len(),
            append_only: true,
            delta_ready: true,
        },
        source_stats: sink.stats,
        route_counts,
        layer_counts,
        outputs: LinuxAtlasOutputs {
            out_dir: config.out_dir.display().to_string(),
            facts_path: facts_path.display().to_string(),
            manifest_path: manifest_path.display().to_string(),
            current_path: current_path.display().to_string(),
            route_index_path: route_index_path.display().to_string(),
            fact_id_index_path: fact_id_index_path.display().to_string(),
            pack_log_path: pack_log_path.display().to_string(),
        },
        sample_facts: sink.facts.iter().take(64).cloned().collect(),
        claim_boundary: LinuxAtlasClaimBoundary {
            linux_atlas_built: fact_count > 0,
            append_only_memory_ready: fact_count > 0,
            active_65k_pack_ready: false,
            exposure_layer_ready: false,
            llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim: "Linux Atlas stores append-only typed Linux facts for later active-field packing. It is not a trained LLM, not an exposure scanner, and not nonlinear-memory proof.",
            blocked_by: vec![
                "linux_active65k_pack_missing",
                "linux_query_eval_missing",
                "linux_exposure_layer_missing",
            ],
        },
    })
}

fn collect_os_release(root: &Path, sink: &mut FactSink) -> Result<()> {
    let path = root.join("etc/os-release");
    let Ok(raw) = fs::read_to_string(&path) else {
        return Ok(());
    };
    for (line_index, line) in raw.lines().enumerate() {
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim_matches('"');
            if matches!(key, "ID" | "VERSION_ID" | "PRETTY_NAME") {
                sink.add(fact(
                    "linux.identity",
                    "os-release",
                    (
                        "linux system",
                        key.to_ascii_lowercase().replace('_', " "),
                        value,
                    ),
                    evidence("os-release", &path, line_index + 1, "os-release"),
                    "positive",
                    95,
                ));
                sink.stats.runtime_snapshot_facts += 1;
            }
        }
    }
    Ok(())
}

fn collect_dpkg_status(root: &Path, sink: &mut FactSink) -> Result<()> {
    let path = root.join("var/lib/dpkg/status");
    let Ok(raw) = fs::read_to_string(&path) else {
        return Ok(());
    };
    for stanza in raw.split("\n\n") {
        if sink.is_full() {
            break;
        }
        let fields = parse_stanza(stanza);
        let Some(package) = fields.get("Package") else {
            continue;
        };
        sink.stats.packages_seen += 1;
        if let Some(version) = fields.get("Version") {
            sink.add(fact(
                "linux.package.version",
                "package",
                (package, "has version", version),
                evidence("dpkg-status", &path, 0, "dpkg-status"),
                "positive",
                90,
            ));
            sink.stats.package_status_facts += 1;
        }
        for relation in ["Depends", "Pre-Depends", "Recommends"] {
            if let Some(value) = fields.get(relation) {
                for dependency in split_dependencies(value).into_iter().take(24) {
                    sink.add(fact(
                        "linux.package.dependency",
                        "package",
                        (
                            package,
                            relation.to_ascii_lowercase().replace('-', " "),
                            &dependency,
                        ),
                        evidence("dpkg-status", &path, 0, "dpkg-status"),
                        "positive",
                        85,
                    ));
                    sink.stats.package_status_facts += 1;
                }
            }
        }
        if let Some(description) = fields.get("Description") {
            if let Some(summary) = description.lines().next() {
                sink.add(fact(
                    "linux.package.summary",
                    "package",
                    (package, "summary", summary),
                    evidence("dpkg-status", &path, 0, "dpkg-status"),
                    "positive",
                    70,
                ));
                sink.stats.package_status_facts += 1;
            }
        }
    }
    Ok(())
}

fn collect_dpkg_file_lists(root: &Path, sink: &mut FactSink) -> Result<()> {
    let dir = root.join("var/lib/dpkg/info");
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(());
    };
    let mut paths = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("list"))
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        if sink.is_full() {
            break;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let package = file_name.trim_end_matches(".list");
        let file = fs::File::open(&path).with_context(|| format!("open {}", path.display()))?;
        for (line_index, line) in BufReader::new(file).lines().enumerate() {
            if sink.is_full() {
                break;
            }
            let line = line.with_context(|| format!("read {}", path.display()))?;
            let Some((route, relation)) = route_for_installed_path(&line) else {
                continue;
            };
            sink.add(fact(
                route,
                "package-file",
                (package, relation, &line),
                evidence("dpkg-list", &path, line_index + 1, "dpkg-file-list"),
                "positive",
                82,
            ));
            sink.stats.package_file_facts += 1;
        }
    }
    Ok(())
}

fn collect_apt_package_lists(root: &Path, sink: &mut FactSink) -> Result<()> {
    let dir = root.join("var/lib/apt/lists");
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(());
    };
    let mut paths = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.ends_with("_Packages") || name.ends_with("_Packages.diff_Index"))
                .unwrap_or(false)
        })
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.ends_with("_Packages"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        if sink.is_full() {
            break;
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        for stanza in raw.split("\n\n") {
            if sink.is_full() {
                break;
            }
            let fields = parse_stanza(stanza);
            let Some(package) = fields.get("Package") else {
                continue;
            };
            sink.stats.apt_packages_seen += 1;
            if let Some(version) = fields.get("Version") {
                sink.add(fact(
                    "linux.apt.package.version",
                    "apt-package-index",
                    (package, "has repository version", version),
                    evidence("apt-packages", &path, 0, "apt-packages"),
                    "positive",
                    84,
                ));
                sink.stats.apt_index_facts += 1;
            }
            if let Some(architecture) = fields.get("Architecture") {
                sink.add(fact(
                    "linux.apt.package.architecture",
                    "apt-package-index",
                    (package, "has architecture", architecture),
                    evidence("apt-packages", &path, 0, "apt-packages"),
                    "positive",
                    80,
                ));
                sink.stats.apt_index_facts += 1;
            }
            if let Some(source) = fields.get("Source") {
                let source_name = source.split_whitespace().next().unwrap_or(source);
                sink.add(fact(
                    "linux.apt.package.source",
                    "apt-package-index",
                    (package, "built from source package", source_name),
                    evidence("apt-packages", &path, 0, "apt-packages"),
                    "positive",
                    80,
                ));
                sink.stats.apt_index_facts += 1;
            }
            if let Some(filename) = fields.get("Filename") {
                sink.add(fact(
                    "linux.apt.package.filename",
                    "apt-package-index",
                    (package, "download path", filename),
                    evidence("apt-packages", &path, 0, "apt-packages"),
                    "positive",
                    78,
                ));
                sink.stats.apt_index_facts += 1;
            }
            if let Some(depends) = fields.get("Depends") {
                for dependency in split_dependencies(depends).into_iter().take(16) {
                    sink.add(fact(
                        "linux.apt.package.dependency",
                        "apt-package-index",
                        (package, "repository depends", &dependency),
                        evidence("apt-packages", &path, 0, "apt-packages"),
                        "positive",
                        76,
                    ));
                    sink.stats.apt_index_facts += 1;
                }
            }
            if let Some(description) = fields.get("Description") {
                if let Some(summary) = description.lines().next() {
                    sink.add(fact(
                        "linux.apt.package.summary",
                        "apt-package-index",
                        (package, "repository summary", summary),
                        evidence("apt-packages", &path, 0, "apt-packages"),
                        "positive",
                        70,
                    ));
                    sink.stats.apt_index_facts += 1;
                }
            }
        }
    }
    Ok(())
}

fn collect_apt_command_indexes(root: &Path, sink: &mut FactSink) -> Result<()> {
    let dir = root.join("var/lib/apt/lists");
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(());
    };
    let mut paths = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.ends_with("_cnf_Commands-amd64"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        if sink.is_full() {
            break;
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        for stanza in raw.split("\n\n") {
            if sink.is_full() {
                break;
            }
            let fields = parse_stanza(stanza);
            let Some(package) = fields.get("name") else {
                continue;
            };
            if let Some(version) = fields.get("version") {
                sink.add(fact(
                    "linux.apt.command.package-version",
                    "apt-command-index",
                    (package, "command index version", version),
                    evidence("apt-command-index", &path, 0, "apt-command-index"),
                    "positive",
                    75,
                ));
                sink.stats.command_index_facts += 1;
            }
            let Some(commands) = fields.get("commands") else {
                continue;
            };
            for command in commands
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
            {
                sink.add(fact(
                    "linux.apt.command.provider",
                    "apt-command-index",
                    (command, "provided by package", package),
                    evidence("apt-command-index", &path, 0, "apt-command-index"),
                    "positive",
                    82,
                ));
                sink.add(fact(
                    "linux.apt.command.package-command",
                    "apt-command-index",
                    (package, "provides command", command),
                    evidence("apt-command-index", &path, 0, "apt-command-index"),
                    "positive",
                    82,
                ));
                sink.stats.command_index_facts += 2;
            }
        }
    }
    Ok(())
}

fn collect_apt_translations(root: &Path, sink: &mut FactSink) -> Result<()> {
    let dir = root.join("var/lib/apt/lists");
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(());
    };
    let mut paths = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.ends_with("_i18n_Translation-en"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        if sink.is_full() {
            break;
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        for stanza in raw.split("\n\n") {
            if sink.is_full() {
                break;
            }
            let fields = parse_stanza(stanza);
            let Some(package) = fields.get("Package") else {
                continue;
            };
            if let Some(description) = fields.get("Description-en") {
                if let Some(summary) = description.lines().next() {
                    sink.add(fact(
                        "linux.apt.translation.summary",
                        "apt-translation-index",
                        (package, "english package description", summary),
                        evidence("apt-translation", &path, 0, "apt-translation"),
                        "positive",
                        68,
                    ));
                    sink.stats.translation_facts += 1;
                }
            }
            if let Some(md5) = fields.get("Description-md5") {
                sink.add(fact(
                    "linux.apt.translation.description-hash",
                    "apt-translation-index",
                    (package, "description hash", md5),
                    evidence("apt-translation", &path, 0, "apt-translation"),
                    "positive",
                    60,
                ));
                sink.stats.translation_facts += 1;
            }
        }
    }
    Ok(())
}

fn collect_manpage_index(root: &Path, sink: &mut FactSink) -> Result<()> {
    let man_root = root.join("usr/share/man");
    collect_manpage_index_inner(&man_root, sink)
}

fn collect_manpage_index_inner(path: &Path, sink: &mut FactSink) -> Result<()> {
    let Ok(entries) = fs::read_dir(path) else {
        return Ok(());
    };
    let mut entries = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect::<Vec<_>>();
    entries.sort();
    for entry_path in entries {
        if sink.is_full() {
            break;
        }
        let Ok(file_type) = fs::metadata(&entry_path).map(|metadata| metadata.file_type()) else {
            continue;
        };
        if file_type.is_dir() {
            collect_manpage_index_inner(&entry_path, sink)?;
            continue;
        }
        let Some(name) = entry_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some((page, section)) = parse_manpage_name(name) else {
            continue;
        };
        sink.add(fact(
            "linux.manpage.index",
            "manual",
            (&page, "documented in section", &section),
            evidence("manpage-index", &entry_path, 0, "manpage-index"),
            "positive",
            78,
        ));
        sink.stats.manpage_facts += 1;
    }
    Ok(())
}

fn collect_systemd_units(root: &Path, sink: &mut FactSink) -> Result<()> {
    for rel in [
        "usr/lib/systemd/system",
        "lib/systemd/system",
        "etc/systemd/system",
    ] {
        collect_systemd_units_inner(root, &root.join(rel), sink)?;
    }
    Ok(())
}

fn collect_systemd_units_inner(root: &Path, path: &Path, sink: &mut FactSink) -> Result<()> {
    let Ok(entries) = fs::read_dir(path) else {
        return Ok(());
    };
    let mut entries = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect::<Vec<_>>();
    entries.sort();
    for entry_path in entries {
        if sink.is_full() {
            break;
        }
        let Ok(metadata) = fs::symlink_metadata(&entry_path) else {
            continue;
        };
        if metadata.is_dir() {
            collect_systemd_units_inner(root, &entry_path, sink)?;
            continue;
        }
        let Some(name) = entry_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !(name.ends_with(".service")
            || name.ends_with(".socket")
            || name.ends_with(".timer")
            || name.ends_with(".target"))
        {
            continue;
        }
        if metadata.len() > 128 * 1024 {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&entry_path) else {
            continue;
        };
        sink.stats.systemd_unit_files += 1;
        for (line_index, line) in raw.lines().enumerate() {
            if sink.is_full() {
                break;
            }
            let trimmed = line.trim();
            if trimmed.starts_with('#') || !trimmed.contains('=') {
                continue;
            }
            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };
            let route = match key {
                "Description" => "linux.systemd.description",
                "After" | "Before" | "Wants" | "Requires" => "linux.systemd.dependency",
                "ExecStart" => "linux.systemd.exec",
                "User" | "Group" => "linux.systemd.identity",
                "WantedBy" | "Alias" => "linux.systemd.install",
                "ListenStream" | "ListenDatagram" => "linux.systemd.socket",
                _ => continue,
            };
            let object = if key == "ExecStart" {
                first_exec_token(value)
            } else {
                value.to_string()
            };
            sink.add(fact(
                route,
                "systemd",
                (name, key.to_ascii_lowercase(), &object),
                evidence(
                    "systemd-unit",
                    entry_path.strip_prefix(root).unwrap_or(&entry_path),
                    line_index + 1,
                    "systemd-unit",
                ),
                "positive",
                88,
            ));
            sink.stats.systemd_facts += 1;
        }
    }
    Ok(())
}

fn collect_runtime_snapshot(root: &Path, sink: &mut FactSink) -> Result<()> {
    collect_resolv_conf(root, sink)?;
    collect_proc_routes(root, sink)?;
    collect_proc_sockets(root, sink)?;
    Ok(())
}

fn collect_resolv_conf(root: &Path, sink: &mut FactSink) -> Result<()> {
    let path = root.join("etc/resolv.conf");
    let Ok(raw) = fs::read_to_string(&path) else {
        return Ok(());
    };
    for (line_index, line) in raw.lines().enumerate() {
        let mut parts = line.split_whitespace();
        if parts.next() != Some("nameserver") {
            continue;
        }
        if let Some(server) = parts.next() {
            sink.add(fact(
                "linux.dns.runtime",
                "runtime-dns",
                ("resolv.conf", "uses nameserver", server),
                evidence("runtime-snapshot", &path, line_index + 1, "resolv-conf"),
                "positive",
                80,
            ));
            sink.stats.runtime_snapshot_facts += 1;
        }
    }
    Ok(())
}

fn collect_proc_routes(root: &Path, sink: &mut FactSink) -> Result<()> {
    let path = root.join("proc/net/route");
    let Ok(raw) = fs::read_to_string(&path) else {
        return Ok(());
    };
    for (line_index, line) in raw.lines().enumerate().skip(1) {
        let cols = line.split_whitespace().collect::<Vec<_>>();
        if cols.len() < 3 {
            continue;
        }
        let iface = cols[0];
        let dest = cols[1];
        let gateway = cols[2];
        sink.add(fact(
            "linux.routing.runtime",
            "runtime-route",
            (
                iface,
                "routes destination",
                &format!("dest={} gateway={}", dest, gateway),
            ),
            evidence("runtime-snapshot", &path, line_index + 1, "proc-route"),
            "positive",
            75,
        ));
        sink.stats.runtime_snapshot_facts += 1;
    }
    Ok(())
}

fn collect_proc_sockets(root: &Path, sink: &mut FactSink) -> Result<()> {
    for rel in [
        "proc/net/tcp",
        "proc/net/tcp6",
        "proc/net/udp",
        "proc/net/udp6",
    ] {
        let path = root.join(rel);
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        for (line_index, line) in raw.lines().enumerate().skip(1) {
            let cols = line.split_whitespace().collect::<Vec<_>>();
            if cols.len() < 4 {
                continue;
            }
            let local = cols[1];
            let state = cols[3];
            if state != "0A" && !rel.contains("udp") {
                continue;
            }
            sink.add(fact(
                "linux.socket.runtime",
                "runtime-socket",
                (
                    rel.trim_start_matches("proc/net/"),
                    if state == "0A" {
                        "listens on"
                    } else {
                        "has socket"
                    },
                    local,
                ),
                evidence("runtime-snapshot", &path, line_index + 1, "proc-socket"),
                "positive",
                72,
            ));
            sink.stats.runtime_snapshot_facts += 1;
        }
    }
    Ok(())
}

fn add_negative_boundary_facts(sink: &mut FactSink) {
    let facts = [
        (
            "linux.boundary.service",
            "systemd service active",
            "does not prove",
            "application is reachable",
        ),
        (
            "linux.boundary.package",
            "package installed",
            "does not prove",
            "binary is running",
        ),
        (
            "linux.boundary.socket",
            "port listening",
            "does not prove",
            "firewall allows external packets",
        ),
        (
            "linux.boundary.vpn",
            "VPN handshake",
            "does not prove",
            "DNS resolution works",
        ),
        (
            "linux.boundary.dns",
            "DNS resolver configured",
            "does not prove",
            "default route is correct",
        ),
        (
            "linux.boundary.container",
            "Dockerfile EXPOSE",
            "does not prove",
            "host port is published",
        ),
        (
            "linux.boundary.cve",
            "vulnerable package",
            "does not prove",
            "runtime exposure",
        ),
    ];
    for (route, subject, relation, object) in facts {
        sink.add(fact(
            route,
            "linux-boundary",
            (subject, relation, object),
            LinuxAtlasEvidence {
                source_kind: "built-in-negative-boundary".to_string(),
                path: "linux-atlas::negative-boundaries".to_string(),
                line: 0,
                extractor: "built-in-boundary".to_string(),
            },
            "negative",
            95,
        ));
        sink.stats.negative_boundary_facts += 1;
    }
}

struct OutputWriteRequest<'a> {
    facts_path: &'a Path,
    manifest_path: &'a Path,
    current_path: &'a Path,
    route_index_path: &'a Path,
    fact_id_index_path: &'a Path,
    pack_log_path: &'a Path,
    facts: &'a [LinuxAtlasFact],
    corpus_hash: &'a str,
    route_counts: &'a BTreeMap<String, usize>,
    layer_counts: &'a BTreeMap<String, usize>,
    known_fact_ids: &'a BTreeSet<String>,
    pack_kind: LinuxAtlasPackKind,
    stats: &'a LinuxAtlasSourceStats,
}

fn write_outputs(request: OutputWriteRequest<'_>) -> Result<()> {
    for path in [
        request.facts_path,
        request.manifest_path,
        request.current_path,
        request.route_index_path,
        request.fact_id_index_path,
        request.pack_log_path,
    ] {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create directory {}", parent.display()))?;
        }
    }

    let mut fact_file = fs::File::create(request.facts_path)
        .with_context(|| format!("create {}", request.facts_path.display()))?;
    for fact in request.facts {
        serde_json::to_writer(&mut fact_file, fact)
            .with_context(|| format!("write fact to {}", request.facts_path.display()))?;
        fact_file
            .write_all(b"\n")
            .with_context(|| format!("finish fact in {}", request.facts_path.display()))?;
    }

    let manifest = serde_json::json!({
        "mode": "linux-atlas-manifest",
        "version": LINUX_ATLAS_VERSION,
        "latest_pack_kind": request.pack_kind.as_str(),
        "latest_facts_path": request.facts_path.display().to_string(),
        "current_path": request.current_path.display().to_string(),
        "fact_id_index_path": request.fact_id_index_path.display().to_string(),
        "pack_log_path": request.pack_log_path.display().to_string(),
        "corpus_hash": request.corpus_hash,
        "fact_count": request.facts.len(),
        "known_fact_count": request.known_fact_ids.len(),
        "route_count": request.route_counts.len(),
        "layer_count": request.layer_counts.len(),
        "append_only": true,
        "source_stats": request.stats,
        "claim_boundary": {
            "linux_atlas_built": !request.facts.is_empty(),
            "active_65k_pack_ready": false,
            "exposure_layer_ready": false,
            "llm_ready": false,
            "nonlinear_memory_proven": false
        }
    });
    write_pretty_json(request.manifest_path, &manifest)?;

    let current = serde_json::json!({
        "mode": "linux-atlas-current",
        "version": LINUX_ATLAS_VERSION,
        "facts_path": request.facts_path.display().to_string(),
        "corpus_hash": request.corpus_hash,
        "fact_count": request.facts.len(),
        "known_fact_count": request.known_fact_ids.len(),
        "route_counts": request.route_counts,
        "layer_counts": request.layer_counts,
        "append_only": true,
        "delta_packs_expected": true
    });
    write_pretty_json(request.current_path, &current)?;
    write_pretty_json(request.route_index_path, request.route_counts)?;
    write_fact_id_index(request.fact_id_index_path, request.known_fact_ids)?;
    append_pack_log(&request)?;
    Ok(())
}

fn write_fact_id_index(path: &Path, fact_ids: &BTreeSet<String>) -> Result<()> {
    let mut file = fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
    for fact_id in fact_ids {
        file.write_all(fact_id.as_bytes())
            .with_context(|| format!("write fact id to {}", path.display()))?;
        file.write_all(b"\n")
            .with_context(|| format!("finish fact id in {}", path.display()))?;
    }
    Ok(())
}

fn append_pack_log(request: &OutputWriteRequest<'_>) -> Result<()> {
    let pack = serde_json::json!({
        "mode": "linux-atlas-pack-log-entry",
        "version": LINUX_ATLAS_VERSION,
        "pack_kind": request.pack_kind.as_str(),
        "facts_path": request.facts_path.display().to_string(),
        "corpus_hash": request.corpus_hash,
        "fact_count": request.facts.len(),
        "known_fact_count": request.known_fact_ids.len(),
        "append_only": true,
        "created_at_unix": unix_timestamp()
    });
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(request.pack_log_path)
        .with_context(|| format!("append {}", request.pack_log_path.display()))?;
    serde_json::to_writer(&mut file, &pack)
        .with_context(|| format!("write {}", request.pack_log_path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish {}", request.pack_log_path.display()))?;
    Ok(())
}

fn load_existing_fact_ids(out_dir: &Path) -> Result<BTreeSet<String>> {
    let index_path = out_dir.join("indexes").join("fact-ids.txt");
    if let Ok(raw) = fs::read_to_string(&index_path) {
        return Ok(raw
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect());
    }

    let facts_dir = out_dir.join("facts");
    let Ok(entries) = fs::read_dir(&facts_dir) else {
        return Ok(BTreeSet::new());
    };
    let mut paths = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("jsonl"))
        .collect::<Vec<_>>();
    paths.sort();

    let mut fact_ids = BTreeSet::new();
    for path in paths {
        let file = fs::File::open(&path).with_context(|| format!("open {}", path.display()))?;
        for line in BufReader::new(file).lines() {
            let line = line.with_context(|| format!("read {}", path.display()))?;
            if line.trim().is_empty() {
                continue;
            }
            let Ok(fact) = serde_json::from_str::<LinuxAtlasFact>(&line) else {
                continue;
            };
            fact_ids.insert(fact.fact_id);
        }
    }
    Ok(fact_ids)
}

fn write_pretty_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
    serde_json::to_writer_pretty(&mut file, value)
        .with_context(|| format!("write {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish {}", path.display()))?;
    Ok(())
}

fn parse_stanza(stanza: &str) -> BTreeMap<String, String> {
    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    let mut current_key: Option<String> = None;
    for line in stanza.lines() {
        if line.starts_with(' ') {
            if let Some(key) = &current_key {
                if let Some(value) = fields.get_mut(key) {
                    value.push('\n');
                    value.push_str(line.trim());
                }
            }
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        current_key = Some(key.to_string());
        fields.insert(key.to_string(), value.trim().to_string());
    }
    fields
}

fn split_dependencies(value: &str) -> Vec<String> {
    value
        .split(',')
        .filter_map(|part| {
            let first = part.split('|').next()?.trim();
            let name = first
                .split_whitespace()
                .next()
                .unwrap_or(first)
                .trim()
                .to_string();
            (!name.is_empty()).then_some(name)
        })
        .collect()
}

fn route_for_installed_path(path: &str) -> Option<(&'static str, &'static str)> {
    if path.starts_with("/usr/bin/") || path.starts_with("/bin/") {
        Some(("linux.package.binary", "provides binary"))
    } else if path.starts_with("/usr/sbin/") || path.starts_with("/sbin/") {
        Some(("linux.package.admin-binary", "provides admin binary"))
    } else if path.starts_with("/etc/") {
        Some(("linux.package.config", "owns config path"))
    } else if path.starts_with("/usr/share/doc/") {
        Some(("linux.package.documentation", "provides documentation path"))
    } else if path.starts_with("/usr/share/applications/") {
        Some(("linux.package.desktop-entry", "provides desktop entry"))
    } else if path.starts_with("/usr/lib/udev/") || path.starts_with("/lib/udev/") {
        Some(("linux.package.udev-rule", "provides udev path"))
    } else if path.starts_with("/usr/share/dbus-1/") {
        Some(("linux.package.dbus", "provides dbus metadata"))
    } else if path.starts_with("/usr/include/") {
        Some(("linux.package.header", "provides header"))
    } else if path.contains("/systemd/system/") {
        Some(("linux.package.systemd-unit", "provides systemd unit"))
    } else if path.starts_with("/usr/share/man/") {
        Some(("linux.package.manpage", "provides manpage"))
    } else if path.starts_with("/usr/lib/") || path.starts_with("/lib/") {
        Some(("linux.package.library", "provides library path"))
    } else {
        None
    }
}

fn parse_manpage_name(name: &str) -> Option<(String, String)> {
    let trimmed = name
        .trim_end_matches(".gz")
        .trim_end_matches(".xz")
        .trim_end_matches(".bz2");
    let (page, section) = trimmed.rsplit_once('.')?;
    if page.is_empty() || section.is_empty() {
        return None;
    }
    Some((page.to_string(), section.to_string()))
}

fn first_exec_token(value: &str) -> String {
    let trimmed = value.trim_start_matches('-').trim();
    let token = trimmed.split_whitespace().next().unwrap_or(trimmed);
    redact_sensitive(token)
}

fn redact_sensitive(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("password")
        || lower.contains("passwd")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("apikey")
        || lower.contains("api_key")
        || lower.contains("privatekey")
    {
        "[REDACTED]".to_string()
    } else {
        value.chars().take(256).collect()
    }
}

fn fact<R>(
    route: &str,
    domain: &str,
    triple: (&str, R, &str),
    evidence: LinuxAtlasEvidence,
    polarity: &str,
    confidence: u8,
) -> LinuxAtlasFact
where
    R: Into<String>,
{
    let (subject, relation, object) = triple;
    LinuxAtlasFact {
        fact_id: String::new(),
        layer: layer_for_route(route).to_string(),
        domain: domain.to_string(),
        route: route.to_string(),
        subject: subject.to_string(),
        relation: relation.into(),
        object: redact_sensitive(object),
        polarity: polarity.to_string(),
        confidence,
        evidence,
    }
}

fn evidence(source_kind: &str, path: &Path, line: usize, extractor: &str) -> LinuxAtlasEvidence {
    LinuxAtlasEvidence {
        source_kind: source_kind.to_string(),
        path: path.display().to_string(),
        line,
        extractor: extractor.to_string(),
    }
}

fn layer_for_route(route: &str) -> &'static str {
    if route.contains("runtime") || route.contains("routing") || route.contains("socket") {
        "runtime-evidence"
    } else if route.contains("boundary") {
        "negative-boundary"
    } else if route.contains("package") || route.contains("manpage") || route.contains("systemd") {
        "linux-knowledge"
    } else {
        "linux-atlas"
    }
}

fn normalize_fact(fact: &mut LinuxAtlasFact) {
    fact.subject = fact.subject.trim().chars().take(256).collect();
    fact.relation = fact.relation.trim().chars().take(96).collect();
    fact.object = fact.object.trim().chars().take(512).collect();
    fact.route = fact.route.trim().to_string();
    fact.domain = fact.domain.trim().to_string();
}

fn fact_key(fact: &LinuxAtlasFact) -> String {
    format!(
        "{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}",
        fact.route, fact.subject, fact.relation, fact.object, fact.polarity
    )
}

fn fact_id(route: &str, key: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(key.as_bytes());
    let digest = format!("{:x}", hash.finalize());
    format!("linux.{}.{}", route.replace('.', "_"), &digest[..16])
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_dependency_split_keeps_first_alternative() {
        assert_eq!(
            split_dependencies("libc6 (>= 2.34), default-mta | mail-transport-agent"),
            vec!["libc6".to_string(), "default-mta".to_string()]
        );
    }

    #[test]
    fn manpage_name_parser_handles_compressed_pages() {
        assert_eq!(
            parse_manpage_name("systemd.service.5.gz"),
            Some(("systemd.service".to_string(), "5".to_string()))
        );
    }
}
