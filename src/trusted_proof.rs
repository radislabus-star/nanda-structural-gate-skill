use crate::*;
use anyhow::bail;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;

pub(crate) const TRUSTED_STRUCTURAL_PROOF_MANIFEST_SCHEMA_V1: &str =
    "nanda.trusted-structural-proof-manifest.v1";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct TrustedStructuralProofManifestV1 {
    pub(crate) schema: String,
    pub(crate) task_id: String,
    pub(crate) engine_core_version: String,
    pub(crate) engine_id: String,
    pub(crate) source_triads_sha256: String,
    pub(crate) candidate_triads_sha256: String,
    pub(crate) candidate_answer_sha256: String,
    pub(crate) source_provenance_root_sha256: String,
    pub(crate) candidate_extraction_root_sha256: String,
    pub(crate) source_producer_root_sha256: String,
    pub(crate) candidate_producer_root_sha256: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TrustedProofValidation {
    pub(crate) mode: String,
    pub(crate) verdict: String,
    pub(crate) authority_ready: bool,
    pub(crate) reason_codes: Vec<String>,
    pub(crate) manifest_file_sha256: String,
    pub(crate) trusted_manifest_root_sha256: String,
    pub(crate) source_triads_sha256: String,
    pub(crate) candidate_triads_sha256: String,
    pub(crate) candidate_answer_sha256: String,
    pub(crate) source_provenance_root_sha256: String,
    pub(crate) candidate_extraction_root_sha256: String,
    pub(crate) source_producer_root_sha256: String,
    pub(crate) candidate_producer_root_sha256: String,
}

impl TrustedProofValidation {
    pub(crate) fn structural_only() -> Self {
        Self {
            mode: "STRUCTURAL_ONLY".to_string(),
            verdict: "NOT_REQUESTED".to_string(),
            authority_ready: false,
            reason_codes: vec!["trusted_proof_not_requested".to_string()],
            manifest_file_sha256: String::new(),
            trusted_manifest_root_sha256: String::new(),
            source_triads_sha256: String::new(),
            candidate_triads_sha256: String::new(),
            candidate_answer_sha256: String::new(),
            source_provenance_root_sha256: String::new(),
            candidate_extraction_root_sha256: String::new(),
            source_producer_root_sha256: String::new(),
            candidate_producer_root_sha256: String::new(),
        }
    }

    fn veto(
        trusted_manifest_root_sha256: &str,
        manifest_file_sha256: String,
        reason: &str,
    ) -> Self {
        Self {
            mode: "TRUSTED_PROOF".to_string(),
            verdict: "VETO".to_string(),
            authority_ready: false,
            reason_codes: vec![reason.to_string()],
            manifest_file_sha256,
            trusted_manifest_root_sha256: trusted_manifest_root_sha256.to_string(),
            source_triads_sha256: String::new(),
            candidate_triads_sha256: String::new(),
            candidate_answer_sha256: String::new(),
            source_provenance_root_sha256: String::new(),
            candidate_extraction_root_sha256: String::new(),
            source_producer_root_sha256: String::new(),
            candidate_producer_root_sha256: String::new(),
        }
    }
}

pub(crate) fn validate_trusted_proof_manifest(
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
    manifest_path: &Path,
    trusted_manifest_root_sha256: &str,
) -> TrustedProofValidation {
    if !is_sha256(trusted_manifest_root_sha256) {
        return TrustedProofValidation::veto(
            trusted_manifest_root_sha256,
            String::new(),
            "invalid_trusted_manifest_root",
        );
    }

    let Ok(manifest_bytes) = fs::read(manifest_path) else {
        return TrustedProofValidation::veto(
            trusted_manifest_root_sha256,
            String::new(),
            "proof_manifest_unreadable",
        );
    };
    let manifest_file_sha256 = sha256_bytes(&manifest_bytes);
    if manifest_file_sha256 != trusted_manifest_root_sha256.to_ascii_lowercase() {
        return TrustedProofValidation::veto(
            trusted_manifest_root_sha256,
            manifest_file_sha256,
            "trusted_manifest_root_mismatch",
        );
    }

    // The external root is checked before parsing so untrusted bytes never gain
    // typed proof status merely by deserializing successfully.
    let Ok(manifest) = serde_json::from_slice::<TrustedStructuralProofManifestV1>(&manifest_bytes)
    else {
        return TrustedProofValidation::veto(
            trusted_manifest_root_sha256,
            manifest_file_sha256,
            "proof_manifest_decode_failed",
        );
    };

    let expected_source_triads_sha256 = triads_sha256(source);
    let expected_candidate_triads_sha256 = triads_sha256(candidates);
    let expected_candidate_answer_sha256 = sha256_bytes(packet.candidate_answer.as_bytes());
    let mut reasons = Vec::new();

    if manifest.schema != TRUSTED_STRUCTURAL_PROOF_MANIFEST_SCHEMA_V1 {
        reasons.push("proof_manifest_schema_mismatch".to_string());
    }
    if manifest.task_id != packet.task_id {
        reasons.push("proof_manifest_task_mismatch".to_string());
    }
    if manifest.engine_core_version != CORE_VERSION || manifest.engine_id != ENGINE_ID {
        reasons.push("proof_manifest_engine_mismatch".to_string());
    }
    if source.is_empty() {
        reasons.push("source_triads_empty".to_string());
    }
    if candidates.is_empty() {
        reasons.push("candidate_triads_empty".to_string());
    }
    if packet.candidate_answer.trim().is_empty() {
        reasons.push("candidate_answer_empty".to_string());
    }
    if manifest.source_triads_sha256 != expected_source_triads_sha256 {
        reasons.push("source_triads_digest_mismatch".to_string());
    }
    if manifest.candidate_triads_sha256 != expected_candidate_triads_sha256 {
        reasons.push("candidate_triads_digest_mismatch".to_string());
    }
    if manifest.candidate_answer_sha256 != expected_candidate_answer_sha256 {
        reasons.push("candidate_answer_digest_mismatch".to_string());
    }

    for (value, reason) in [
        (
            &manifest.source_provenance_root_sha256,
            "invalid_source_provenance_root",
        ),
        (
            &manifest.candidate_extraction_root_sha256,
            "invalid_candidate_extraction_root",
        ),
        (
            &manifest.source_producer_root_sha256,
            "invalid_source_producer_root",
        ),
        (
            &manifest.candidate_producer_root_sha256,
            "invalid_candidate_producer_root",
        ),
    ] {
        if !is_sha256(value) {
            reasons.push(reason.to_string());
        }
    }
    if manifest.source_provenance_root_sha256 == manifest.candidate_extraction_root_sha256 {
        reasons.push("source_candidate_provenance_not_independent".to_string());
    }
    if manifest.source_producer_root_sha256 == manifest.candidate_producer_root_sha256 {
        reasons.push("source_candidate_producer_not_independent".to_string());
    }
    if candidate_reuses_source_evidence(source, candidates) {
        reasons.push("candidate_reuses_source_evidence".to_string());
    }

    reasons.sort();
    reasons.dedup();
    TrustedProofValidation {
        mode: "TRUSTED_PROOF".to_string(),
        verdict: if reasons.is_empty() { "PASS" } else { "VETO" }.to_string(),
        authority_ready: reasons.is_empty(),
        reason_codes: reasons,
        manifest_file_sha256,
        trusted_manifest_root_sha256: trusted_manifest_root_sha256.to_ascii_lowercase(),
        source_triads_sha256: expected_source_triads_sha256,
        candidate_triads_sha256: expected_candidate_triads_sha256,
        candidate_answer_sha256: expected_candidate_answer_sha256,
        source_provenance_root_sha256: manifest.source_provenance_root_sha256,
        candidate_extraction_root_sha256: manifest.candidate_extraction_root_sha256,
        source_producer_root_sha256: manifest.source_producer_root_sha256,
        candidate_producer_root_sha256: manifest.candidate_producer_root_sha256,
    }
}

pub(crate) fn build_proof_manifest_draft(
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
    source_provenance_root_sha256: String,
    candidate_extraction_root_sha256: String,
    source_producer_root_sha256: String,
    candidate_producer_root_sha256: String,
) -> TrustedStructuralProofManifestV1 {
    TrustedStructuralProofManifestV1 {
        schema: TRUSTED_STRUCTURAL_PROOF_MANIFEST_SCHEMA_V1.to_string(),
        task_id: packet.task_id.clone(),
        engine_core_version: CORE_VERSION.to_string(),
        engine_id: ENGINE_ID.to_string(),
        source_triads_sha256: triads_sha256(source),
        candidate_triads_sha256: triads_sha256(candidates),
        candidate_answer_sha256: sha256_bytes(packet.candidate_answer.as_bytes()),
        source_provenance_root_sha256,
        candidate_extraction_root_sha256,
        source_producer_root_sha256,
        candidate_producer_root_sha256,
    }
}

pub(crate) fn proof_manifest_draft_cmd(args: ProofManifestDraftArgs) -> Result<u8> {
    let packet = load_packet(Some(&args.triads))?;
    let source = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    if source.is_empty() || candidates.is_empty() || packet.candidate_answer.trim().is_empty() {
        bail!(
            "proof manifest draft requires source triads, candidate triads, and candidate_answer"
        );
    }
    for (value, label) in [
        (&args.source_provenance_root, "source_provenance_root"),
        (&args.candidate_extraction_root, "candidate_extraction_root"),
        (&args.source_producer_root, "source_producer_root"),
        (&args.candidate_producer_root, "candidate_producer_root"),
    ] {
        if !is_sha256(value) {
            bail!("{label} must be a lowercase SHA-256 digest");
        }
    }
    if args.source_provenance_root == args.candidate_extraction_root {
        bail!("source and candidate provenance roots must differ");
    }
    if args.source_producer_root == args.candidate_producer_root {
        bail!("source and candidate producer roots must differ");
    }

    let manifest = build_proof_manifest_draft(
        &packet,
        &source,
        &candidates,
        args.source_provenance_root,
        args.candidate_extraction_root,
        args.source_producer_root,
        args.candidate_producer_root,
    );
    let bytes = serde_json::to_vec_pretty(&manifest)?;
    let mut bytes_with_newline = bytes;
    bytes_with_newline.push(b'\n');
    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args.out)
        .with_context(|| format!("create proof manifest draft {}", args.out.display()))?;
    file.write_all(&bytes_with_newline)?;
    file.sync_all()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "nanda.proof-manifest-draft-report.v1",
            "manifest_path": args.out,
            "manifest_file_sha256": sha256_bytes(&bytes_with_newline),
            "trust_state": "UNTRUSTED_DRAFT_REQUIRES_EXTERNAL_PIN"
        }))?
    );
    Ok(EXIT_PASS)
}

fn candidate_reuses_source_evidence(source: &[Triad], candidates: &[Triad]) -> bool {
    candidates.iter().any(|candidate| {
        source.iter().any(|source_triad| {
            (!candidate.evidence.trim().is_empty()
                && norm(&candidate.evidence) == norm(&source_triad.evidence))
                || (!candidate.evidence_path.trim().is_empty()
                    && norm(&candidate.evidence_path) == norm(&source_triad.evidence_path))
        })
    })
}

fn triads_sha256(triads: &[Triad]) -> String {
    sha256_bytes(&serde_json::to_vec(triads).expect("triads serialize"))
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

    fn digest(seed: &str) -> String {
        sha256_bytes(seed.as_bytes())
    }

    fn packet(candidate_evidence: &str, candidate_evidence_path: &str) -> Packet {
        serde_json::from_value(json!({
            "task_id": "trusted-proof-test",
            "domain": "code",
            "candidate_answer": "The capture owner seals the support prefix.",
            "triads": [{
                "id": "t1",
                "subject": "capture owner",
                "relation": "seals",
                "object": "support prefix",
                "evidence": "source.rs:10",
                "subject_role": "owner",
                "object_role": "artifact",
                "route": "freeze",
                "group": "source",
                "evidence_path": "source.rs"
            }],
            "candidate_triads": [{
                "id": "c1",
                "subject": "capture owner",
                "relation": "seals",
                "object": "support prefix",
                "evidence": candidate_evidence,
                "subject_role": "owner",
                "object_role": "artifact",
                "route": "freeze",
                "group": "candidate",
                "evidence_path": candidate_evidence_path
            }]
        }))
        .expect("packet")
    }

    fn manifest_for(packet: &Packet) -> TrustedStructuralProofManifestV1 {
        build_proof_manifest_draft(
            packet,
            &normalize_ids(packet.triads.clone(), "t"),
            &normalize_ids(packet.candidate_triads.clone(), "c"),
            digest("source-provenance"),
            digest("candidate-extraction"),
            digest("source-producer"),
            digest("candidate-producer"),
        )
    }

    fn write_manifest(manifest: &TrustedStructuralProofManifestV1) -> (PathBuf, String) {
        let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nanda-trusted-proof-{}-{id}.json",
            std::process::id()
        ));
        let bytes = serde_json::to_vec(manifest).expect("manifest bytes");
        fs::write(&path, &bytes).expect("manifest write");
        (path, sha256_bytes(&bytes))
    }

    fn validate(packet: &Packet, path: &Path, root: &str) -> TrustedProofValidation {
        validate_trusted_proof_manifest(
            packet,
            &normalize_ids(packet.triads.clone(), "t"),
            &normalize_ids(packet.candidate_triads.clone(), "c"),
            path,
            root,
        )
    }

    #[test]
    fn externally_pinned_independent_manifest_enables_authority() {
        let packet = packet("candidate_answer:1", "candidate.json");
        let (path, root) = write_manifest(&manifest_for(&packet));
        let proof = validate(&packet, &path, &root);
        fs::remove_file(path).expect("cleanup");

        assert_eq!(proof.verdict, "PASS");
        assert!(proof.authority_ready);
        assert!(proof.reason_codes.is_empty());
    }

    #[test]
    fn foreign_or_recomputed_manifest_root_is_rejected_before_parse() {
        let packet = packet("candidate_answer:1", "candidate.json");
        let (path, _) = write_manifest(&manifest_for(&packet));
        let proof = validate(&packet, &path, &digest("foreign-root"));
        fs::remove_file(path).expect("cleanup");

        assert_eq!(proof.verdict, "VETO");
        assert_eq!(proof.reason_codes, ["trusted_manifest_root_mismatch"]);
        assert!(!proof.authority_ready);
    }

    #[test]
    fn source_evidence_copy_is_rejected_even_with_a_pinned_manifest() {
        let packet = packet("source.rs:10", "source.rs");
        let (path, root) = write_manifest(&manifest_for(&packet));
        let proof = validate(&packet, &path, &root);
        fs::remove_file(path).expect("cleanup");

        assert_eq!(proof.verdict, "VETO");
        assert!(proof
            .reason_codes
            .contains(&"candidate_reuses_source_evidence".to_string()));
    }

    #[test]
    fn source_evidence_copy_is_rejected_after_structural_key_mutation() {
        let mut packet = packet("source.rs:10", "candidate.json");
        packet.candidate_triads[0].object = "mutated support prefix".to_string();
        let (path, root) = write_manifest(&manifest_for(&packet));
        let proof = validate(&packet, &path, &root);
        fs::remove_file(path).expect("cleanup");

        assert_ne!(
            structural_key(&packet.triads[0]),
            structural_key(&packet.candidate_triads[0])
        );
        assert_eq!(proof.verdict, "VETO");
        assert!(proof
            .reason_codes
            .contains(&"candidate_reuses_source_evidence".to_string()));
    }

    #[test]
    fn candidate_mutation_after_manifest_seal_is_rejected() {
        let original = packet("candidate_answer:1", "candidate.json");
        let (path, root) = write_manifest(&manifest_for(&original));
        let mutated = packet("candidate_answer:2", "candidate.json");
        let proof = validate(&mutated, &path, &root);
        fs::remove_file(path).expect("cleanup");

        assert_eq!(proof.verdict, "VETO");
        assert!(proof
            .reason_codes
            .contains(&"candidate_triads_digest_mismatch".to_string()));
    }

    #[test]
    fn same_producer_or_provenance_roots_are_rejected() {
        let packet = packet("candidate_answer:1", "candidate.json");
        let mut manifest = manifest_for(&packet);
        manifest.candidate_extraction_root_sha256 = manifest.source_provenance_root_sha256.clone();
        manifest.candidate_producer_root_sha256 = manifest.source_producer_root_sha256.clone();
        let (path, root) = write_manifest(&manifest);
        let proof = validate(&packet, &path, &root);
        fs::remove_file(path).expect("cleanup");

        assert_eq!(proof.verdict, "VETO");
        assert!(proof
            .reason_codes
            .contains(&"source_candidate_provenance_not_independent".to_string()));
        assert!(proof
            .reason_codes
            .contains(&"source_candidate_producer_not_independent".to_string()));
    }
}
