pub(super) fn diff_changed_files(diff: &str) -> Vec<String> {
    let mut out = vec![];
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            if rest != "/dev/null" {
                out.push(rest.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("--- a/") {
            if rest != "/dev/null" {
                out.push(rest.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("diff --git a/") {
            if let Some((left, _)) = rest.split_once(" b/") {
                out.push(left.to_string());
            }
        }
    }
    out
}

pub(super) fn diff_changed_functions(diff: &str) -> Vec<String> {
    let mut current_file = None::<String>;
    let mut out = vec![];
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("diff --git a/") {
            current_file = rest.split_once(" b/").map(|(left, _)| left.to_string());
            continue;
        }
        if let Some(hunk) = line.strip_prefix("@@") {
            if let Some(file) = current_file.as_deref() {
                let symbol = hunk
                    .rsplit_once("@@")
                    .map(|(_, after)| after.trim())
                    .filter(|after| !after.is_empty())
                    .unwrap_or("hunk");
                out.push(format!("{file}::{symbol}"));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

pub(super) fn diff_added_public_api(diff: &str) -> Vec<String> {
    diff_added_lines(diff)
        .into_iter()
        .filter(|(_, line)| is_public_api_line(line))
        .map(|(file, line)| format!("{file}:{}", line.trim()))
        .collect()
}

pub(super) fn diff_runtime_side_effects(diff: &str) -> Vec<String> {
    diff_added_lines(diff)
        .into_iter()
        .filter(|(_, line)| is_runtime_side_effect_line(line))
        .map(|(file, line)| format!("{file}:{}", line.trim()))
        .collect()
}

fn diff_added_lines(diff: &str) -> Vec<(String, String)> {
    let mut current_file = None::<String>;
    let mut out = vec![];
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("diff --git a/") {
            current_file = rest.split_once(" b/").map(|(left, _)| left.to_string());
            continue;
        }
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        if let (Some(file), Some(added)) = (current_file.as_ref(), line.strip_prefix('+')) {
            out.push((file.clone(), added.to_string()));
        }
    }
    out
}

fn is_public_api_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("pub ")
        || trimmed.starts_with("pub(")
        || trimmed.starts_with("pub(crate)")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("pub(crate) fn ")
}

fn is_runtime_side_effect_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "std::process::command",
        "command::new",
        "fs::write",
        "file::create",
        "remove_file",
        "remove_dir",
        "rename(",
        "systemctl",
        "service ",
        "dbus",
        "spawn(",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

pub(super) fn is_test_file(file: &str) -> bool {
    file.contains("/tests/")
        || file.starts_with("tests/")
        || file.contains("_test")
        || file.contains("test_")
        || file.ends_with(".spec.js")
}

pub(super) fn file_from_diff_ref(item: &str) -> &str {
    item.split_once(':').map_or(item, |(file, _)| file)
}

pub(super) fn path_matches(changed: &str, allowed: &str) -> bool {
    changed == allowed || changed.starts_with(allowed) || allowed.starts_with(changed)
}
