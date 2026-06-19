use crate::*;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct FocusCacheRecord {
    pub(crate) key: String,
    pub(crate) packet_path: PathBuf,
    pub(crate) manifest_path: PathBuf,
}

pub(crate) fn cache_cmd(command: CacheCommand) -> Result<u8> {
    match command {
        CacheCommand::Build(args) => cache_build_cmd(args),
        CacheCommand::List(args) => cache_list_cmd(args),
    }
}

pub(crate) fn cache_record(
    cache_dir: &Path,
    packet: &Packet,
    query_text: &str,
    query_source: &str,
    max_triads: usize,
    route_cap: usize,
    route_triad_cap: usize,
) -> Result<FocusCacheRecord> {
    let key = focus_cache_key(
        packet,
        query_text,
        query_source,
        max_triads,
        route_cap,
        route_triad_cap,
    )?;
    Ok(FocusCacheRecord {
        packet_path: cache_dir.join(format!("{key}.focus.json")),
        manifest_path: cache_dir.join(format!("{key}.manifest.json")),
        key,
    })
}

pub(crate) fn load_focus_cache(record: &FocusCacheRecord) -> Result<Option<focus::FocusBuild>> {
    if !record.packet_path.exists() || !record.manifest_path.exists() {
        return Ok(None);
    }
    let packet_text = fs::read_to_string(&record.packet_path)
        .with_context(|| format!("read {}", record.packet_path.display()))?;
    let packet: Packet = serde_json::from_str(&packet_text)
        .with_context(|| format!("parse {}", record.packet_path.display()))?;
    let manifest_text = fs::read_to_string(&record.manifest_path)
        .with_context(|| format!("read {}", record.manifest_path.display()))?;
    let metadata: Value = serde_json::from_str(&manifest_text)
        .with_context(|| format!("parse {}", record.manifest_path.display()))?;
    Ok(Some(focus::FocusBuild { packet, metadata }))
}

pub(crate) fn load_focus_cache_manifest(path: &Path) -> Result<(focus::FocusBuild, Value)> {
    let manifest_path = resolve_manifest_path(path)?;
    let manifest_text = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let metadata: Value = serde_json::from_str(&manifest_text)
        .with_context(|| format!("parse {}", manifest_path.display()))?;
    let packet_path = cache_packet_path(&metadata, &manifest_path)?;
    let packet_text = fs::read_to_string(&packet_path)
        .with_context(|| format!("read {}", packet_path.display()))?;
    let packet: Packet = serde_json::from_str(&packet_text)
        .with_context(|| format!("parse {}", packet_path.display()))?;
    let cache_report = json!({
        "enabled": true,
        "state": "CACHE_ONLY_HIT",
        "key": metadata["cache"]["key"],
        "packet_path": packet_path.display().to_string(),
        "manifest_path": manifest_path.display().to_string(),
        "source": metadata["cache"]["source"]
    });
    Ok((focus::FocusBuild { packet, metadata }, cache_report))
}

fn resolve_manifest_path(path: &Path) -> Result<PathBuf> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    if !path.is_dir() {
        return Err(anyhow::anyhow!(
            "cache-only path is neither file nor directory: {}",
            path.display()
        ));
    }
    let mut manifests = fs::read_dir(path)
        .with_context(|| format!("read {}", path.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".manifest.json"))
        })
        .collect::<Vec<_>>();
    manifests.sort();
    match manifests.len() {
        1 => Ok(manifests.remove(0)),
        0 => Err(anyhow::anyhow!(
            "cache-only directory has no *.manifest.json: {}",
            path.display()
        )),
        count => Err(anyhow::anyhow!(
            "cache-only directory has {count} manifests; pass a specific manifest path"
        )),
    }
}

fn cache_packet_path(metadata: &Value, manifest_path: &Path) -> Result<PathBuf> {
    let packet = metadata["cache"]["packet_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("cache manifest is missing cache.packet_path"))?;
    let path = PathBuf::from(packet);
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path))
    }
}

pub(crate) fn write_focus_cache(
    record: &FocusCacheRecord,
    build: &focus::FocusBuild,
    source: &str,
) -> Result<Value> {
    fs::create_dir_all(
        record
            .packet_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new(".")),
    )
    .with_context(|| {
        format!(
            "create {}",
            record
                .packet_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .display()
        )
    })?;
    fs::write(
        &record.packet_path,
        serde_json::to_string_pretty(&build.packet)? + "\n",
    )
    .with_context(|| format!("write {}", record.packet_path.display()))?;
    let mut metadata = build.metadata.clone();
    metadata["cache"] = json!({
        "key": record.key,
        "source": source,
        "packet_path": record.packet_path.display().to_string(),
        "manifest_path": record.manifest_path.display().to_string()
    });
    fs::write(
        &record.manifest_path,
        serde_json::to_string_pretty(&metadata)? + "\n",
    )
    .with_context(|| format!("write {}", record.manifest_path.display()))?;
    Ok(metadata["cache"].clone())
}

fn cache_build_cmd(args: CacheBuildArgs) -> Result<u8> {
    let mut packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let mut query_packet = if let Some(query_file) = &args.query_file {
        load_packet_auto(
            query_file,
            &args.query_format,
            &args.task_id,
            &args.domain,
            &args.query,
            args.normalize_paths,
        )?
    } else {
        packet.clone()
    };
    inherit_aliases_if_needed(&mut query_packet, &packet);
    let query_text = if !args.query.trim().is_empty() {
        args.query.clone()
    } else if !query_packet.query.trim().is_empty() {
        query_packet.query.clone()
    } else {
        packet.query.clone()
    };
    packet.query = query_text.clone();
    let (query, query_source) = search_query_triads(&query_packet, &query_text);
    let memory = normalize_ids(packet.triads.clone(), "m");
    let build = focus::build_focused_packet(
        &packet,
        &memory,
        &query,
        query_source,
        args.max_triads,
        args.route_cap,
        args.route_triad_cap,
    );
    let record = cache_record(
        &args.out_dir,
        &packet,
        &query_text,
        query_source,
        args.max_triads,
        args.route_cap,
        args.route_triad_cap,
    )?;
    let cache = write_focus_cache(&record, &build, "nanda-cache build")?;
    let out = json!({
        "core_version": CORE_VERSION,
        "mode": "focus-cache-build",
        "version": "v64-focus-cache",
        "cache": cache,
        "input_memory_size": packet.triads.len(),
        "focused_memory_size": build.packet.triads.len(),
        "focused_query_size": build.packet.candidate_triads.len(),
        "focus": build.metadata,
        "read_as": "Reusable focused packet cache for large-corpus fast proof. Cache keys include packet content, query source, query text, and focus caps."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_cache_text(&out),
        OutputFormat::Md => print_cache_md(&out),
    }
    Ok(EXIT_PASS)
}

fn cache_list_cmd(args: CacheListArgs) -> Result<u8> {
    let mut records = vec![];
    if args.dir.exists() {
        for entry in
            fs::read_dir(&args.dir).with_context(|| format!("read {}", args.dir.display()))?
        {
            let path = entry?.path();
            let is_manifest = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".manifest.json"));
            if !is_manifest {
                continue;
            }
            let text =
                fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
            let metadata: Value =
                serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
            records.push(json!({
                "key": metadata["cache"]["key"],
                "manifest_path": path.display().to_string(),
                "packet_path": metadata["cache"]["packet_path"],
                "source": metadata["cache"]["source"],
                "original_memory_size": metadata["original_memory_size"],
                "focused_memory_size": metadata["focused_memory_size"],
                "focused_routes": metadata["focused_routes"],
                "query_source": metadata["query_source"],
                "runtime_state": metadata["runtime_contract"]["state"]
            }));
        }
    }
    records.sort_by(|a, b| {
        a["key"]
            .as_str()
            .unwrap_or("")
            .cmp(b["key"].as_str().unwrap_or(""))
    });
    let out = json!({
        "core_version": CORE_VERSION,
        "mode": "focus-cache-list",
        "version": "v65-cache-only-proof",
        "dir": args.dir.display().to_string(),
        "count": records.len(),
        "records": records,
        "read_as": "List reusable focused packets available for cache-only proof."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_cache_list_text(&out),
        OutputFormat::Md => print_cache_list_md(&out),
    }
    Ok(EXIT_PASS)
}

fn focus_cache_key(
    packet: &Packet,
    query_text: &str,
    query_source: &str,
    max_triads: usize,
    route_cap: usize,
    route_triad_cap: usize,
) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(CORE_VERSION.as_bytes());
    hasher.update(nanda_6m::VERSION.as_bytes());
    hasher.update(query_text.as_bytes());
    hasher.update(query_source.as_bytes());
    hasher.update(max_triads.to_le_bytes());
    hasher.update(route_cap.to_le_bytes());
    hasher.update(route_triad_cap.to_le_bytes());
    hasher.update(serde_json::to_vec(packet)?);
    let digest = hasher.finalize();
    Ok(digest
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>())
}

fn print_cache_text(out: &Value) {
    println!("NANDA FOCUS CACHE");
    println!("key: {}", out["cache"]["key"].as_str().unwrap_or(""));
    println!(
        "focus: {} -> {} triads",
        out["input_memory_size"].as_u64().unwrap_or(0),
        out["focused_memory_size"].as_u64().unwrap_or(0)
    );
    println!(
        "packet: {}",
        out["cache"]["packet_path"].as_str().unwrap_or("")
    );
}

fn print_cache_md(out: &Value) {
    println!("# NANDA Focus Cache\n");
    println!("- key: `{}`", out["cache"]["key"].as_str().unwrap_or(""));
    println!(
        "- focus: `{}` -> `{}` triads",
        out["input_memory_size"].as_u64().unwrap_or(0),
        out["focused_memory_size"].as_u64().unwrap_or(0)
    );
    println!(
        "- packet: `{}`",
        out["cache"]["packet_path"].as_str().unwrap_or("")
    );
}

fn print_cache_list_text(out: &Value) {
    println!("NANDA FOCUS CACHE LIST");
    println!("dir: {}", out["dir"].as_str().unwrap_or(""));
    println!("count: {}", out["count"].as_u64().unwrap_or(0));
    if let Some(records) = out["records"].as_array() {
        for record in records {
            println!(
                "- {} focus={} runtime={}",
                record["key"].as_str().unwrap_or(""),
                record["focused_memory_size"].as_u64().unwrap_or(0),
                record["runtime_state"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_cache_list_md(out: &Value) {
    println!("# NANDA Focus Cache List\n");
    println!("- dir: `{}`", out["dir"].as_str().unwrap_or(""));
    println!("- count: `{}`", out["count"].as_u64().unwrap_or(0));
    if let Some(records) = out["records"].as_array() {
        for record in records {
            println!(
                "- `{}` focus=`{}` runtime=`{}`",
                record["key"].as_str().unwrap_or(""),
                record["focused_memory_size"].as_u64().unwrap_or(0),
                record["runtime_state"].as_str().unwrap_or("")
            );
        }
    }
}
