use serde_json::Value;

pub(super) fn sample(items: &[String], cap: usize) -> Vec<String> {
    items.iter().take(cap).cloned().collect()
}

pub(super) fn sample_values(items: &[Value], cap: usize) -> Vec<Value> {
    items.iter().take(cap).cloned().collect()
}
