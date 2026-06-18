#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
packet="${1:-"$root/.nanda/finance-16k-risk-cluster.json"}"

node "$root/scripts/generate-finance-16k-fixture.js" "$packet" >/tmp/nanda-heavy-16k-generate.json
jq -e '.triads == 16384 and .candidate_triads == 3' /tmp/nanda-heavy-16k-generate.json >/dev/null
jq -e '.triads | length == 16384' "$packet" >/dev/null

doctor_json="$(nanda-dataset-doctor "$packet" --input-format json)"
jq -e '.triad_count == 16384' <<<"$doctor_json" >/dev/null
jq -e '.verdict == "PASS"' <<<"$doctor_json" >/dev/null
jq -e '.notices[]? | select(.kind == "large_but_route_balanced_focus")' <<<"$doctor_json" >/dev/null

set +e
pack_json="$(nanda-pack6m "$packet" --input-format json --sample 1)"
pack_status=$?
set -e
if [[ "$pack_status" -ne 3 ]]; then
  echo "expected full 16k pack6m WATCH exit status 3, got $pack_status" >&2
  echo "$pack_json" >&2
  exit 1
fi
jq -e '.packed_records.memory_count == 16384' <<<"$pack_json" >/dev/null
jq -e '.peak_decision.state == "PACKED_FOCUSED"' <<<"$pack_json" >/dev/null
jq -e '.runtime_contract.state == "FOCUS_REQUIRED"' <<<"$pack_json" >/dev/null
jq -e '.runtime_contract.ready == false' <<<"$pack_json" >/dev/null

set +e
search_json="$(nanda-search "$packet" --input-format json --route-cap 64 --route-triad-cap 2000 --top-k 5)"
search_status=$?
set -e
if [[ "$search_status" -ne 0 ]]; then
  echo "expected full 16k search command to complete, got $search_status" >&2
  echo "$search_json" >&2
  exit 1
fi
jq -e '.route_balanced_focus.focused_memory_size == 16384' <<<"$search_json" >/dev/null
jq -e '.verdict == "WATCH"' <<<"$search_json" >/dev/null
jq -e '.top_peak == "ai-demand"' <<<"$search_json" >/dev/null
jq -e '.field_state == "FIELD_CONTESTED"' <<<"$search_json" >/dev/null
jq -e '.safe_to_answer == false' <<<"$search_json" >/dev/null
jq -e '.resonant_field.version == "v28-resonant-field-core"' <<<"$search_json" >/dev/null
jq -e '.resonant_field.waw_status == "NO_WAW_RESONANCE"' <<<"$search_json" >/dev/null
jq -e '.resonant_field.phase_lock.state == "PHASE_PARTIAL"' <<<"$search_json" >/dev/null
jq -e '.resonant_field.standing_wave.state == "STANDING_UNSTABLE"' <<<"$search_json" >/dev/null

set +e
proof_json="$(nanda-proof "$packet" --input-format json --max-triads 15000 --route-cap 32 --route-triad-cap 600 --format json)"
proof_status=$?
set -e
if [[ "$proof_status" -ne 3 ]]; then
  echo "expected focused proof WATCH exit status 3, got $proof_status" >&2
  echo "$proof_json" >&2
  exit 1
fi
jq -e '.input_memory_size == 16384' <<<"$proof_json" >/dev/null
jq -e '.focused_memory_size == 9620' <<<"$proof_json" >/dev/null
jq -e '.runtime_ready == true' <<<"$proof_json" >/dev/null
jq -e '.proof_state == "WATCH"' <<<"$proof_json" >/dev/null
jq -e '.field_state == "FIELD_CONTESTED"' <<<"$proof_json" >/dev/null
jq -e '.reason_codes | index("FIELD_CONTESTED")' <<<"$proof_json" >/dev/null
jq -e '.reason_codes | index("RESONANCE_FIELD_DIFFUSE")' <<<"$proof_json" >/dev/null
jq -e '.hot_proof.packed_peak_state == "PACKED_FOCUSED"' <<<"$proof_json" >/dev/null

echo "ok"
