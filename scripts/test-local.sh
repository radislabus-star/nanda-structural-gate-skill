#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
checker="$root/nanda-structural-gate/scripts/nanda-check"
gate="$root/nanda-structural-gate/scripts/nanda-gate"
init_task="$root/nanda-structural-gate/scripts/nanda-init-task"
pack_md="$root/nanda-structural-gate/scripts/nanda-pack-from-md"
init_md="$root/nanda-structural-gate/scripts/nanda-init-md"
gate_md="$root/nanda-structural-gate/scripts/nanda-gate-md"
self_check="$root/nanda-structural-gate/scripts/nanda-self-check"
comb="$root/nanda-structural-gate/scripts/nanda-comb"
doctor="$root/nanda-structural-gate/scripts/nanda-doctor"
extractor="$root/nanda-structural-gate/scripts/nanda-extract"
evaler="$root/nanda-structural-gate/scripts/nanda-eval"
feedback="$root/nanda-structural-gate/scripts/nanda-feedback"
indexer="$root/nanda-structural-gate/scripts/nanda-index"
search="$root/nanda-structural-gate/scripts/nanda-search"
dogfood="$root/nanda-structural-gate/scripts/nanda-dogfood"
reporter="$root/nanda-structural-gate/scripts/nanda-report"
split_md="$root/nanda-structural-gate/scripts/nanda-split-md"
split_packet="$root/nanda-structural-gate/scripts/nanda-split"
mapper="$root/nanda-structural-gate/scripts/nanda-map"

cargo fmt --check --manifest-path "$root/Cargo.toml"
cargo check --manifest-path "$root/Cargo.toml" >/dev/null
cargo test --manifest-path "$root/Cargo.toml" >/dev/null
jq empty "$root/examples/triad-packet.example.json"
jq empty "$root/examples/triad-packet.role-swap.json"
jq empty "$root/examples/triad-packet.route-splice.json"
jq empty "$root/examples/triad-packet.evidence-conflict.json"
jq empty "$root/examples/triad-packet.watch-missing-evidence.json"
jq empty "$root/examples/triad-packet.watch-low-complexity.json"
jq empty "$root/examples/triad-packet.interference-search.json"
jq empty "$root/examples/triad-packet.interference-search-noisy.json"
jq empty "$root/examples/triad-packet.interference-search-route-trap.json"

pass_json="$("$checker" --triads "$root/examples/triad-packet.example.json" --format json)"
if ! grep -q '"verdict": "PASS"' <<<"$pass_json"; then
  echo "expected PASS example" >&2
  echo "$pass_json" >&2
  exit 1
fi

set +e
veto_json="$("$checker" --triads "$root/examples/triad-packet.role-swap.json" --format json)"
veto_status=$?
set -e
if [[ "$veto_status" -eq 0 ]]; then
  echo "expected VETO exit status" >&2
  echo "$veto_json" >&2
  exit 1
fi
if ! grep -q '"verdict": "VETO"' <<<"$veto_json"; then
  echo "expected VETO example" >&2
  echo "$veto_json" >&2
  exit 1
fi

set +e
watch_json="$("$checker" --triads "$root/examples/triad-packet.watch-missing-evidence.json" --format json)"
watch_status=$?
set -e
if [[ "$watch_status" -ne 3 ]]; then
  echo "expected WATCH exit status 3" >&2
  echo "$watch_json" >&2
  exit 1
fi
if ! grep -q '"verdict": "WATCH"' <<<"$watch_json"; then
  echo "expected WATCH example" >&2
  echo "$watch_json" >&2
  exit 1
fi

"$gate" --triads "$root/examples/triad-packet.example.json" >/dev/null
set +e
"$gate" --triads "$root/examples/triad-packet.watch-missing-evidence.json" >/dev/null 2>&1
gate_watch_status=$?
set -e
if [[ "$gate_watch_status" -ne 3 ]]; then
  echo "expected nanda-gate WATCH block" >&2
  exit 1
fi

tmp_task="$(mktemp)"
"$init_task" --task-id smoke --domain contract --query "smoke" --out "$tmp_task" >/dev/null
jq empty "$tmp_task"
rm -f "$tmp_task"

tmp_md_packet="$(mktemp)"
"$pack_md" "$root/examples/triads.route-splice.md" --task-id md-splice --domain contract --out "$tmp_md_packet" >/dev/null
jq empty "$tmp_md_packet"
set +e
"$checker" --triads "$tmp_md_packet" >/dev/null
md_status=$?
set -e
rm -f "$tmp_md_packet"
if [[ "$md_status" -ne 1 ]]; then
  echo "expected Markdown-packed route splice to VETO" >&2
  exit 1
fi

tmp_norm_packet="$(mktemp)"
"$pack_md" "$root/examples/triads.code-path-normalization.md" --task-id path-normalize --domain code --normalize-paths --out "$tmp_norm_packet" >/dev/null
jq empty "$tmp_norm_packet"
grep -q '"subject": "core::gate"' "$tmp_norm_packet"
grep -q '"object": "bin::nanda"' "$tmp_norm_packet"
"$checker" --triads "$tmp_norm_packet" >/dev/null
rm -f "$tmp_norm_packet"

tmp_md="$(mktemp)"
"$init_md" --task-id md-smoke --domain code --query "smoke" --template code --out "$tmp_md" >/dev/null
test -s "$tmp_md"
rm -f "$tmp_md"

set +e
"$gate_md" "$root/examples/triads.route-splice.md" --task-id md-gate --domain contract >/dev/null 2>&1
gate_md_status=$?
set -e
if [[ "$gate_md_status" -ne 1 ]]; then
  echo "expected nanda-gate-md route splice to VETO" >&2
  exit 1
fi

"$gate_md" "$root/examples/triads.code-flow.md" --task-id code-flow --domain code >/dev/null
set +e
"$gate_md" "$root/examples/triads.code-flow-splice.md" --task-id code-flow-splice --domain code >/dev/null 2>&1
code_splice_status=$?
set -e
if [[ "$code_splice_status" -ne 1 ]]; then
  echo "expected code flow splice to VETO" >&2
  exit 1
fi

map_json="$("$mapper" "$root/examples/triads.code-flow-splice.md" --task-id code-map --domain code)"
grep -q '"core_version": "sparse-triad-v1.0-release"' <<<"$map_json"
grep -q '"wave_dim": 1024' <<<"$map_json"
grep -q '"mixed_candidate_groups"' <<<"$map_json"
grep -q '"candidate-code-flow"' <<<"$map_json"
grep -q '"group_centroids"' <<<"$map_json"
grep -q '"route_memory"' <<<"$map_json"
grep -q '"candidate_superposition"' <<<"$map_json"
grep -q '"foreign_pull"' <<<"$map_json"

tmp_split="$(mktemp -d)"
"$split_md" "$root/examples/triads.code-flow-splice.md" --by group --out-dir "$tmp_split" >/dev/null
test -s "$tmp_split/triads.code-flow-splice-group-candidate-code-flow.md"
test -s "$tmp_split/triads.code-flow-splice-group-flow-a.md"
test -s "$tmp_split/triads.code-flow-splice-group-flow-b.md"
rm -rf "$tmp_split"

tmp_linked_split="$(mktemp -d)"
linked_out="$("$split_md" "$root/examples/triads.linked-group-split.md" --by linked-group --out-dir "$tmp_linked_split")"
grep -q '"mode": "linked-group"' <<<"$linked_out"
test -s "$tmp_linked_split/triads.linked-group-split-linked-group-flow-a.md"
grep -q 'source module A' "$tmp_linked_split/triads.linked-group-split-linked-group-flow-a.md"
grep -q 'candidate-flow-a' "$tmp_linked_split/triads.linked-group-split-linked-group-flow-a.md"
if grep -q 'source group flow-a has no linked candidate group' <<<"$linked_out"; then
  echo "linked-group split produced source-only warning for paired flow" >&2
  echo "$linked_out" >&2
  exit 1
fi
rm -rf "$tmp_linked_split"

tmp_json_packet="$(mktemp)"
"$pack_md" "$root/examples/triads.linked-group-split.md" --task-id linked-json --domain code --out "$tmp_json_packet" >/dev/null
tmp_json_split="$(mktemp -d)"
json_split_out="$("$split_packet" "$tmp_json_packet" --input-format json --by linked-group --out-dir "$tmp_json_split")"
grep -q '"mode": "linked-group"' <<<"$json_split_out"
grep -q '"format": "json"' <<<"$json_split_out"
json_file="$(find "$tmp_json_split" -type f -name '*.json' | head -n 1)"
test -n "$json_file"
jq empty "$json_file"
grep -q '"triads"' "$json_file"
grep -q '"candidate_triads"' "$json_file"
"$checker" --triads "$json_file" >/dev/null
rm -f "$tmp_json_packet"
rm -rf "$tmp_json_split"

comb_json="$("$comb" "$root/examples/triads.code-flow-splice.md" --domain code --depth 1)"
grep -q '"topology"' <<<"$comb_json"
grep -q '"comb_tree"' <<<"$comb_json"
grep -q '"foreign_pull"' <<<"$comb_json"

drift_packet="$(mktemp)"
"$pack_md" "$root/examples/triads.invariant-drift.md" --task-id invariant-drift --domain code --out "$drift_packet" >/dev/null
drift_comb="$("$comb" "$drift_packet" --input-format json --depth 2)"
grep -q '"invariant_violation"' <<<"$drift_comb"
grep -q '"setting.timeout"' <<<"$drift_comb"
grep -q '"300ms"' <<<"$drift_comb"
grep -q '"500ms"' <<<"$drift_comb"
rm -f "$drift_packet"

search_json="$("$search" "$root/examples/triad-packet.interference-search.json" --input-format json --top-k 3)"
grep -q '"mode": "interference-retrieval"' <<<"$search_json"
grep -q '"peak": "certification"' <<<"$search_json"
first_peak="$(jq -r '.peaks[0].peak' <<<"$search_json")"
if [[ "$first_peak" != "certification" ]]; then
  echo "expected certification as top interference peak, got $first_peak" >&2
  echo "$search_json" >&2
  exit 1
fi
grep -q '"supporting_triads"' <<<"$search_json"
grep -q '"anti_triads"' <<<"$search_json"
grep -q 'customs declaration' <<<"$search_json"
search_text="$("$search" "$root/examples/triad-packet.interference-search.json" --input-format json --format text --top-k 1)"
grep -q 'peak=certification' <<<"$search_text"
noisy_search_json="$("$search" "$root/examples/triad-packet.interference-search-noisy.json" --input-format json --top-k 3)"
noisy_first_peak="$(jq -r '.peaks[0].peak' <<<"$noisy_search_json")"
if [[ "$noisy_first_peak" != "certification" ]]; then
  echo "expected noisy query to focus certification peak, got $noisy_first_peak" >&2
  echo "$noisy_search_json" >&2
  exit 1
fi
grep -q '"peak_margin"' <<<"$noisy_search_json"
grep -q '"lexical_baseline"' <<<"$noisy_search_json"
grep -q '"symbolic_baseline"' <<<"$noisy_search_json"
grep -q '"anti_triads"' <<<"$noisy_search_json"
jq -e '.peak_decision.state == "WATCH"' <<<"$noisy_search_json" >/dev/null
jq -e '.peak_decision.safe_to_answer == false' <<<"$noisy_search_json" >/dev/null
eval_json="$("$evaler" \
  --case "$root/examples/triad-packet.interference-search-route-trap.json:certification:FOCUSED" \
  --case "$root/examples/triad-packet.interference-search-noisy.json:certification:WATCH")"
jq -e '.mode == "eval-suite"' <<<"$eval_json" >/dev/null
jq -e '.passed == 2 and .total == 2 and .accuracy == 1' <<<"$eval_json" >/dev/null
doctor_json="$("$doctor")"
jq -e '.mode == "doctor" and .healthy == true' <<<"$doctor_json" >/dev/null
jq -e '.route_trap.top == "certification" and .route_trap.state == "FOCUSED"' <<<"$doctor_json" >/dev/null
jq -e '.noisy.state == "WATCH" and .noisy.safe_to_answer == false' <<<"$doctor_json" >/dev/null
trap_search_json="$("$search" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --top-k 3)"
trap_first_peak="$(jq -r '.peaks[0].peak' <<<"$trap_search_json")"
trap_lexical_peak="$(jq -r '.lexical_baseline.top_peak' <<<"$trap_search_json")"
trap_wins="$(jq -r '.wins_over_lexical_baseline' <<<"$trap_search_json")"
if [[ "$trap_first_peak" != "certification" || "$trap_lexical_peak" != "customs" || "$trap_wins" != "true" ]]; then
  echo "expected interference peak to beat lexical route trap" >&2
  echo "$trap_search_json" >&2
  exit 1
fi
jq -e '.peak_margin > 0.05' <<<"$trap_search_json" >/dev/null
jq -e '.peaks[0].propagation.component_score > .peaks[1].propagation.component_score' <<<"$trap_search_json" >/dev/null
jq -e '.peak_decision.state == "FOCUSED"' <<<"$trap_search_json" >/dev/null
jq -e '.peak_decision.safe_to_answer == true' <<<"$trap_search_json" >/dev/null
tmp_search="$(mktemp)"
tmp_feedback="$(mktemp)"
printf '%s\n' "$trap_search_json" >"$tmp_search"
"$feedback" "$tmp_search" --decision accept --note "route trap accepted" --out "$tmp_feedback" >/dev/null
jq empty "$tmp_feedback"
jq -e '.mode == "feedback-memory"' "$tmp_feedback" >/dev/null
jq -e '.decision == "accept"' "$tmp_feedback" >/dev/null
jq -e '.peak == "certification"' "$tmp_feedback" >/dev/null
jq -e '.memory_patch.reinforce_peak == "certification"' "$tmp_feedback" >/dev/null
rm -f "$tmp_search" "$tmp_feedback"
tmp_index="$(mktemp)"
"$indexer" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --out "$tmp_index" >/dev/null
jq empty "$tmp_index"
indexed_search_json="$("$search" "$tmp_index" --input-format json --query-file "$root/examples/triad-packet.interference-search-route-trap.json" --query-format json --top-k 3)"
jq -e '.peak_decision.state == "FOCUSED"' <<<"$indexed_search_json" >/dev/null
jq -e '.wins_over_lexical_baseline == true' <<<"$indexed_search_json" >/dev/null
rm -f "$tmp_index"
tmp_extract="$(mktemp)"
"$extractor" "$root/examples/route-trap.raw.txt" --out "$tmp_extract" >/dev/null
jq empty "$tmp_extract"
extracted_search_json="$("$search" "$tmp_extract" --input-format json --top-k 3)"
jq -e '.peak_decision.state == "FOCUSED"' <<<"$extracted_search_json" >/dev/null
jq -e '.lexical_baseline.top_peak == "customs"' <<<"$extracted_search_json" >/dev/null
rm -f "$tmp_extract"

dogfood_json="$("$dogfood" "$root" --format json)"
grep -q '"mode": "dogfood"' <<<"$dogfood_json"
grep -q '"action": "SAFE_TO_EDIT"' <<<"$dogfood_json"
grep -q '"root_verdict": "WATCH"' <<<"$dogfood_json"
grep -q '"root_size_only": true' <<<"$dogfood_json"
grep -q '"foreign_pull": 0' <<<"$dogfood_json"
grep -q '"invariant_violation": 0' <<<"$dogfood_json"
grep -q '"local_branches": 7' <<<"$dogfood_json"
grep -q '"local_pass": 7' <<<"$dogfood_json"
dogfood_text="$("$dogfood" "$root")"
grep -q 'ACTION: SAFE_TO_EDIT' <<<"$dogfood_text"
grep -q 'BRANCHES: 7/7 PASS' <<<"$dogfood_text"

"$init_md" --task-id skill-smoke --template skill --stdout >/dev/null
"$init_md" --task-id project-smoke --template project --stdout >/dev/null
"$doctor" --help | grep -q "Usage: nanda doctor"
"$evaler" --help | grep -q "Usage: nanda eval"
"$feedback" --help | grep -q "Usage: nanda feedback"
NANDA_SELF_CHECK_RUNTIME_ONLY=1 "$self_check" | grep -q "verdict: PASS"

set +e
report_out="$("$reporter" \
  --title "Smoke Report" \
  --domain contract \
  --overall "$root/examples/triads.watch-large.md" \
  --route invoice:"$root/examples/triads.route-splice.md" 2>/dev/null)"
report_status=$?
set -e
if [[ "$report_status" -ne 1 ]]; then
  echo "expected nanda-report route-splice VETO status" >&2
  echo "$report_out" >&2
  exit 1
fi
grep -q '"action": "REPAIR_REQUIRED"' <<<"$report_out"

set +e
report_watch="$("$reporter" \
  --title "Smoke Watch Report" \
  --domain code \
  --overall "$root/examples/triads.watch-large.md" \
  --route code:"$root/examples/triads.code-flow.md" 2>/dev/null)"
report_watch_status=$?
set -e
if [[ "$report_watch_status" -ne 3 ]]; then
  echo "expected nanda-report overall WATCH status" >&2
  echo "$report_watch" >&2
  exit 1
fi
grep -q '"action": "DRAFT_OK_REVIEW_REQUIRED"' <<<"$report_watch"
grep -q '"safe_to_draft": true' <<<"$report_watch"
grep -q '"safe_to_send": false' <<<"$report_watch"
"$gate_md" "$root/examples/triads.skill-flow.md" --task-id skill-flow --domain skill >/dev/null
set +e
"$gate_md" "$root/examples/triads.skill-flow-splice.md" --task-id skill-flow-splice --domain skill >/dev/null 2>&1
skill_splice_status=$?
set -e
if [[ "$skill_splice_status" -ne 1 ]]; then
  echo "expected skill flow splice to VETO" >&2
  exit 1
fi

"$root/scripts/benchmark-v0.sh" >/dev/null
"$root/scripts/test-edge-cases.sh" >/dev/null
"$gate_md" --help | grep -q "Usage: nanda gate-md"
"$reporter" --help | grep -q "Usage: nanda report"
"$mapper" --help | grep -q "Usage: nanda map"
"$comb" --help | grep -q "Usage: nanda comb"
"$extractor" --help | grep -q "Usage: nanda extract"
"$indexer" --help | grep -q "Usage: nanda index"
"$search" --help | grep -q "Usage: nanda search"
"$dogfood" --help | grep -q "Usage: nanda dogfood"
"$split_packet" --help | grep -q "Usage: nanda split"
"$reporter" --format md --title "Smoke Markdown Report" --domain code --overall "$root/examples/triads.watch-large.md" --route code:"$root/examples/triads.code-flow.md" >/dev/null || test "$?" -eq 3

echo "ok"
