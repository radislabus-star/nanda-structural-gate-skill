#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
toolchain="${NANDA_RUST_TOOLCHAIN:-1.97.0}"
binary="$root/target/debug/nanda"
fixture="$root/examples/triad-packet.trusted-proof.json"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cargo "+$toolchain" build --quiet --manifest-path "$root/Cargo.toml"

structural_json="$($binary check --triads "$fixture" --format json)"
jq -e '.verdict == "PASS" and .authority_ready == false and .trusted_proof.verdict == "NOT_REQUESTED"' \
  <<<"$structural_json" >/dev/null

source_provenance_root="$(printf source-provenance | sha256sum | cut -d' ' -f1)"
candidate_extraction_root="$(printf candidate-extraction | sha256sum | cut -d' ' -f1)"
source_producer_root="$(printf source-producer | sha256sum | cut -d' ' -f1)"
candidate_producer_root="$(printf candidate-producer | sha256sum | cut -d' ' -f1)"

draft_json="$($binary proof-manifest-draft \
  --triads "$fixture" \
  --source-provenance-root "$source_provenance_root" \
  --candidate-extraction-root "$candidate_extraction_root" \
  --source-producer-root "$source_producer_root" \
  --candidate-producer-root "$candidate_producer_root" \
  --out "$tmp/manifest.json")"
manifest_root="$(jq -r .manifest_file_sha256 <<<"$draft_json")"
jq -e '.trust_state == "UNTRUSTED_DRAFT_REQUIRES_EXTERNAL_PIN"' <<<"$draft_json" >/dev/null

trusted_json="$($binary check \
  --triads "$fixture" \
  --proof-manifest "$tmp/manifest.json" \
  --trusted-manifest-root "$manifest_root" \
  --format json)"
jq -e '.verdict == "PASS" and .authority_ready == true and .trusted_proof.verdict == "PASS"' \
  <<<"$trusted_json" >/dev/null

set +e
foreign_json="$($binary check \
  --triads "$fixture" \
  --proof-manifest "$tmp/manifest.json" \
  --trusted-manifest-root "$(printf foreign-root | sha256sum | cut -d' ' -f1)" \
  --format json)"
foreign_status=$?
set -e
[[ "$foreign_status" -eq 1 ]]
jq -e '.verdict == "VETO" and .authority_ready == false and (.trusted_proof.reason_codes | index("trusted_manifest_root_mismatch"))' \
  <<<"$foreign_json" >/dev/null

jq '.candidate_triads[0].evidence = .triads[0].evidence |
    .candidate_triads[0].evidence_path = .triads[0].evidence_path |
    .candidate_triads[0].object = "mutated support prefix"' \
  "$fixture" >"$tmp/copied.json"
copied_draft="$($binary proof-manifest-draft \
  --triads "$tmp/copied.json" \
  --source-provenance-root "$source_provenance_root" \
  --candidate-extraction-root "$candidate_extraction_root" \
  --source-producer-root "$source_producer_root" \
  --candidate-producer-root "$candidate_producer_root" \
  --out "$tmp/copied-manifest.json")"
copied_root="$(jq -r .manifest_file_sha256 <<<"$copied_draft")"
set +e
copied_json="$($binary check \
  --triads "$tmp/copied.json" \
  --proof-manifest "$tmp/copied-manifest.json" \
  --trusted-manifest-root "$copied_root" \
  --format json)"
copied_status=$?
set -e
[[ "$copied_status" -eq 1 ]]
jq -e '.verdict == "VETO" and .authority_ready == false and (.trusted_proof.reason_codes | index("candidate_reuses_source_evidence"))' \
  <<<"$copied_json" >/dev/null

set +e
empty_json="$($binary check \
  --triads "$root/examples/triad-packet.empty-candidate-high-complexity.json" \
  --format json)"
empty_status=$?
set -e
[[ "$empty_status" -eq 3 ]]
jq -e '.verdict == "WATCH" and .complexity_score >= 12 and .authority_ready == false' \
  <<<"$empty_json" >/dev/null

echo "ok"
