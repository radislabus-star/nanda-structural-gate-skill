use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChatCoreSlot {
    Route,
    Subject,
    Object,
    FreeText,
}

#[derive(Serialize, Clone, Debug)]
pub(crate) struct ChatCoreSecretScan {
    pub secret_detected: bool,
    pub secret_refused: bool,
    pub review_or_quarantine: bool,
    pub reasons: Vec<String>,
}

pub(crate) fn scan_feedback_slots(raw: &str) -> ChatCoreSecretScan {
    if let Some(parts) = split_pipe_fact(raw) {
        return merge_scans([
            scan_slot_for_secret(ChatCoreSlot::Subject, parts[0]),
            scan_slot_for_secret(ChatCoreSlot::Route, parts[1]),
            scan_slot_for_secret(ChatCoreSlot::Object, parts[2]),
        ]);
    }
    if let Some((left, right)) = raw.split_once("implies") {
        return merge_scans([
            scan_slot_for_secret(ChatCoreSlot::Subject, left),
            scan_slot_for_secret(ChatCoreSlot::Object, right),
            scan_slot_for_secret(ChatCoreSlot::FreeText, raw),
        ]);
    }
    scan_slot_for_secret(ChatCoreSlot::FreeText, raw)
}

pub(crate) fn scan_slot_for_secret(slot: ChatCoreSlot, value: &str) -> ChatCoreSecretScan {
    let lower = value.to_ascii_lowercase();
    let mut reasons = Vec::new();
    for marker in [
        "-----begin",
        "private key",
        "token=",
        "access_token",
        "api_key",
        "apikey",
        "secret=",
        "client_secret",
        "password=",
        "passwd=",
        "authorization:",
        "bearer ",
        "sk-",
        "ghp_",
        "github_pat_",
        "xoxb-",
        "xoxp-",
        "akia",
    ] {
        if lower.contains(marker) {
            reasons.push(format!("secret_marker:{marker}"));
        }
    }
    let secret_markers_found = !reasons.is_empty();
    if matches!(slot, ChatCoreSlot::FreeText) && looks_like_jwt(value) {
        reasons.push("jwt_like_token".to_string());
    }
    let mut review_or_quarantine = false;
    if matches!(slot, ChatCoreSlot::Subject | ChatCoreSlot::Object)
        && !secret_markers_found
        && long_technical_identifier(value)
    {
        review_or_quarantine = true;
        reasons.push("long_technical_identifier_review".to_string());
    }
    if matches!(slot, ChatCoreSlot::FreeText) && has_long_high_entropy_token(value) {
        reasons.push("long_high_entropy_token".to_string());
    }
    let secret_detected = reasons
        .iter()
        .any(|reason| reason.starts_with("secret_marker:") || reason == "jwt_like_token")
        || (matches!(slot, ChatCoreSlot::FreeText)
            && reasons
                .iter()
                .any(|reason| reason == "long_high_entropy_token"));
    ChatCoreSecretScan {
        secret_detected,
        secret_refused: secret_detected,
        review_or_quarantine,
        reasons,
    }
}

fn merge_scans(scans: impl IntoIterator<Item = ChatCoreSecretScan>) -> ChatCoreSecretScan {
    let mut merged = ChatCoreSecretScan {
        secret_detected: false,
        secret_refused: false,
        review_or_quarantine: false,
        reasons: Vec::new(),
    };
    for scan in scans {
        merged.secret_detected |= scan.secret_detected;
        merged.secret_refused |= scan.secret_refused;
        merged.review_or_quarantine |= scan.review_or_quarantine;
        merged.reasons.extend(scan.reasons);
    }
    merged
}

fn split_pipe_fact(raw: &str) -> Option<Vec<&str>> {
    let parts = raw.split('|').map(str::trim).collect::<Vec<_>>();
    (parts.len() == 3 && parts.iter().all(|part| !part.is_empty())).then_some(parts)
}

fn looks_like_jwt(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts.iter().all(|part| part.len() >= 10)
        && parts.iter().all(|part| {
            part.bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        })
}

fn long_technical_identifier(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() >= 48
        && trimmed.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-' || byte == b'.'
        })
}

fn has_long_high_entropy_token(value: &str) -> bool {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' && ch != '+')
        .any(|part| part.len() >= 48 && high_entropyish(part))
}

fn high_entropyish(value: &str) -> bool {
    let has_upper = value.bytes().any(|byte| byte.is_ascii_uppercase());
    let has_lower = value.bytes().any(|byte| byte.is_ascii_lowercase());
    let has_digit = value.bytes().any(|byte| byte.is_ascii_digit());
    let has_symbol = value.contains(['_', '-', '+']);
    [has_upper, has_lower, has_digit, has_symbol]
        .into_iter()
        .filter(|present| *present)
        .count()
        >= 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_like_identifier_is_not_jwt_secret() {
        let scan = scan_slot_for_secret(
            ChatCoreSlot::Route,
            "physics.material_layer.candidate_status",
        );
        assert!(!scan.secret_detected);
        assert!(!scan.secret_refused);
    }

    #[test]
    fn long_snake_case_identifier_is_review_not_hard_secret() {
        let scan = scan_slot_for_secret(
            ChatCoreSlot::Subject,
            "compact_high_r2_weather_transfer_unclassified_by_v1",
        );
        assert!(!scan.secret_detected);
        assert!(!scan.secret_refused);
        assert!(scan.review_or_quarantine);
    }

    #[test]
    fn secret_marker_still_refuses() {
        let scan = scan_feedback_slots(
            "foocmd | linux.apt.command.package-command | sk-live-secret-token",
        );
        assert!(scan.secret_detected);
        assert!(scan.secret_refused);
    }
}
