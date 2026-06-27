use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn check_version_bump_contract(atlas: &Value, diff: &str, changed: &[String]) -> Value {
    let repo = atlas["repo"]
        .as_str()
        .or_else(|| atlas["input"].as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let scope_violations = changed
        .iter()
        .filter(|file| !is_version_owned_file(&repo, file, diff))
        .cloned()
        .collect::<Vec<_>>();
    let cargo_package = parse_cargo_toml_package(&repo.join("Cargo.toml"));
    let cargo_package_name = cargo_package.as_ref().map(|package| package.0.as_str());
    let cargo_version = cargo_package.as_ref().map(|package| package.1.clone());
    let mut checks = vec![];
    let mut violations = vec![];

    if let Some(version) = cargo_version.as_deref() {
        checks.push(json!({"name": "cargo_toml_package_version", "ok": true, "value": version}));
    } else {
        violations.push("Cargo.toml package.version not found".to_string());
        checks.push(json!({"name": "cargo_toml_package_version", "ok": false}));
    }

    if changed.iter().any(|file| file == "Cargo.lock") {
        match (
            cargo_package_name,
            cargo_version.as_deref(),
            cargo_package_name
                .and_then(|name| parse_cargo_lock_package_version(&repo.join("Cargo.lock"), name)),
        ) {
            (Some(package), Some(expected), Some(actual)) if expected == actual => {
                checks.push(json!({"name": "cargo_lock_package_version", "ok": true, "package": package, "value": actual}));
            }
            (Some(package), Some(expected), Some(actual)) => {
                violations.push(format!(
                    "Cargo.lock package {package} version {actual} != Cargo.toml {expected}"
                ));
                checks.push(json!({"name": "cargo_lock_package_version", "ok": false, "package": package, "expected": expected, "actual": actual}));
            }
            (Some(package), _, None) => {
                violations.push(format!("Cargo.lock package {package} version not found"));
                checks.push(
                    json!({"name": "cargo_lock_package_version", "ok": false, "package": package}),
                );
            }
            (Some(package), None, Some(actual)) => {
                violations.push(format!(
                    "Cargo.toml package {package} version not found while Cargo.lock has {actual}"
                ));
                checks.push(
                    json!({"name": "cargo_lock_package_version", "ok": false, "package": package, "actual": actual}),
                );
            }
            (None, _, _) => {
                violations.push("Cargo.toml package.name not found".to_string());
                checks.push(json!({"name": "cargo_lock_package_version", "ok": false}));
            }
        }
    }

    for file in changed
        .iter()
        .filter(|file| file.ends_with("metadata.json"))
    {
        match (cargo_version.as_deref(), read_json_file(&repo.join(file))) {
            (Some(expected), Some(json_value)) => {
                let name_ok = json_value["version-name"].as_str() == Some(expected);
                let numeric_ok = json_value["version"]
                    .as_i64()
                    .is_some_and(|value| version_number_matches(expected, value));
                if !name_ok {
                    violations.push(format!("{file} version-name does not match {expected}"));
                }
                if !numeric_ok {
                    violations.push(format!(
                        "{file} numeric version does not match patch/build for {expected}"
                    ));
                }
                checks.push(json!({
                    "name": "metadata_json_version",
                    "file": file,
                    "ok": name_ok && numeric_ok,
                    "version_name": json_value["version-name"].clone(),
                    "numeric_version": json_value["version"].clone()
                }));
            }
            _ => {
                violations.push(format!("{file} metadata.json unreadable"));
                checks.push(json!({"name": "metadata_json_version", "file": file, "ok": false}));
            }
        }
    }

    for file in changed.iter().filter(|file| is_js_version_file(file)) {
        let versions = parse_app_versions(&repo.join(file));
        if versions.is_empty() {
            continue;
        }
        let file_ok = cargo_version
            .as_deref()
            .is_some_and(|expected| versions.iter().all(|actual| actual == expected));
        if !file_ok {
            violations.push(format!("{file} APP_VERSION does not match Cargo.toml"));
        }
        checks.push(json!({
            "name": "js_app_version",
            "file": file,
            "ok": file_ok,
            "versions": versions
        }));
    }

    if let Some(expected) = cargo_version.as_deref() {
        for file in changed
            .iter()
            .filter(|file| should_scan_for_stale_version(file))
        {
            let stale = stale_version_tokens_for_file(file, &repo.join(file), expected);
            if !stale.is_empty() {
                violations.push(format!(
                    "{file} still contains stale version(s): {}",
                    stale.join(", ")
                ));
                checks.push(
                    json!({"name": "stale_version_scan", "file": file, "ok": false, "stale": stale}),
                );
            }
        }
    }

    json!({
        "contract": "shared.version_bump_contract",
        "contract_scope": "version metadata only",
        "scope_ok": scope_violations.is_empty(),
        "consistent": scope_violations.is_empty() && violations.is_empty() && cargo_version.is_some(),
        "cargo_package": cargo_package_name,
        "cargo_version": cargo_version,
        "scope_violations": scope_violations,
        "violations": violations,
        "checks": checks
    })
}

fn is_version_owned_file(repo: &Path, file: &str, diff: &str) -> bool {
    matches!(file, "Cargo.toml" | "Cargo.lock" | "VERSIONING.md")
        || (file.starts_with("extension/")
            && matches!(
                Path::new(file).file_name().and_then(|name| name.to_str()),
                Some("metadata.json" | "prefs.js" | "settings.js" | "tray_support.js")
            ))
        || (matches!(file, "README.md" | "HOW_IT_WORKS.md")
            && file_or_diff_contains_version(&repo.join(file), file, diff))
}

fn file_or_diff_contains_version(path: &Path, file: &str, diff: &str) -> bool {
    fs::read_to_string(path)
        .map(|content| contains_semver(&content))
        .unwrap_or(false)
        || diff_block_contains_semver(file, diff)
}

fn diff_block_contains_semver(file: &str, diff: &str) -> bool {
    let mut in_file = false;
    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("diff --git a/") {
            let file_marker = format!(" b/{file}");
            in_file = path.ends_with(&file_marker);
            continue;
        }
        if in_file && contains_semver(line) {
            return true;
        }
    }
    false
}

fn parse_cargo_toml_version(path: &Path) -> Option<String> {
    parse_cargo_toml_package(path).map(|package| package.1)
}

fn parse_cargo_toml_package(path: &Path) -> Option<(String, String)> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_package = false;
    let mut name = None;
    let mut version = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            if !in_package && name.is_some() && version.is_some() {
                break;
            }
        } else if in_package && trimmed.starts_with("name") {
            name = quoted_value(trimmed);
        } else if in_package && trimmed.starts_with("version") {
            version = quoted_value(trimmed);
        }
    }
    Some((name?, version?))
}

fn parse_cargo_lock_package_version(path: &Path, package_name: &str) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_package = false;
    let mut is_target_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            if in_package && is_target_package {
                return None;
            }
            in_package = true;
            is_target_package = false;
            continue;
        }
        if !in_package {
            continue;
        }
        if trimmed.starts_with("name") && quoted_value(trimmed).as_deref() == Some(package_name) {
            is_target_package = true;
        } else if is_target_package && trimmed.starts_with("version") {
            return quoted_value(trimmed);
        }
    }
    None
}

fn quoted_value(line: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let mut quoted = line.split(quote);
        let _before = quoted.next()?;
        if let Some(value) = quoted.next() {
            return Some(value.trim().to_string()).filter(|value| !value.is_empty());
        }
    }
    let (_, value) = line.split_once('=')?;
    Some(
        value
            .trim()
            .trim_end_matches(';')
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string(),
    )
    .filter(|value| !value.is_empty())
}

fn read_json_file(path: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

fn version_number_matches(version: &str, value: i64) -> bool {
    let parts = version
        .split('.')
        .filter_map(|part| part.parse::<i64>().ok())
        .collect::<Vec<_>>();
    if parts.len() != 3 {
        return false;
    }
    let patch = parts[2];
    let packed = parts[0] * 10_000 + parts[1] * 100 + parts[2];
    value == patch || value == packed
}

fn is_js_version_file(file: &str) -> bool {
    matches!(
        Path::new(file).file_name().and_then(|name| name.to_str()),
        Some("prefs.js" | "settings.js" | "tray_support.js")
    )
}

fn parse_app_versions(path: &Path) -> Vec<String> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return vec![],
    };
    content
        .lines()
        .filter_map(parse_app_version_definition)
        .collect()
}

fn should_scan_for_stale_version(file: &str) -> bool {
    file == "Cargo.toml" || file.ends_with("metadata.json") || is_js_version_file(file)
}

fn parse_app_version_definition(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed
        .strip_prefix("export ")
        .unwrap_or(trimmed)
        .trim_start();
    let rest = rest.strip_prefix("const ")?.trim_start();
    let rest = rest.strip_prefix("APP_VERSION")?.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value = rest[quote.len_utf8()..].split(quote).next()?.trim();
    Some(value.to_string()).filter(|value| !value.is_empty())
}

fn stale_version_tokens_for_file(file: &str, path: &Path, expected: &str) -> Vec<String> {
    if file == "Cargo.toml" {
        return parse_cargo_toml_version(path)
            .filter(|version| version != expected)
            .into_iter()
            .collect();
    }
    if file.ends_with("metadata.json") {
        return read_json_file(path)
            .and_then(|json_value| json_value["version-name"].as_str().map(str::to_string))
            .filter(|version| version != expected)
            .into_iter()
            .collect();
    }
    if is_js_version_file(file) {
        return parse_app_versions(path)
            .into_iter()
            .filter(|version| version != expected)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
    }
    vec![]
}

fn contains_semver(value: &str) -> bool {
    !semver_tokens(value).is_empty()
}

fn semver_tokens(value: &str) -> Vec<String> {
    value
        .split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .filter(|part| {
            let pieces = part.split('.').collect::<Vec<_>>();
            pieces.len() == 3
                && pieces
                    .iter()
                    .all(|piece| !piece.is_empty() && piece.chars().all(|ch| ch.is_ascii_digit()))
        })
        .map(str::to_string)
        .collect()
}
