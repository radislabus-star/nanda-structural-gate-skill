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
waw="$root/nanda-structural-gate/scripts/nanda-waw"
feedback="$root/nanda-structural-gate/scripts/nanda-feedback"
indexer="$root/nanda-structural-gate/scripts/nanda-index"
search="$root/nanda-structural-gate/scripts/nanda-search"
focus="$root/nanda-structural-gate/scripts/nanda-focus"
probe="$root/nanda-structural-gate/scripts/nanda-probe"
dataset_doctor="$root/nanda-structural-gate/scripts/nanda-dataset-doctor"
aliases="$root/nanda-structural-gate/scripts/nanda-aliases"
budget="$root/nanda-structural-gate/scripts/nanda-budget"
pack6m="$root/nanda-structural-gate/scripts/nanda-pack6m"
bench6m="$root/nanda-structural-gate/scripts/nanda-bench6m"
serve="$root/nanda-structural-gate/scripts/nanda-serve"
dogfood="$root/nanda-structural-gate/scripts/nanda-dogfood"
reporter="$root/nanda-structural-gate/scripts/nanda-report"
split_md="$root/nanda-structural-gate/scripts/nanda-split-md"
split_packet="$root/nanda-structural-gate/scripts/nanda-split"
mapper="$root/nanda-structural-gate/scripts/nanda-map"
code_mapper="$root/nanda-structural-gate/scripts/nanda-map-code"

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
jq empty "$root/examples/triad-packet.waw-code-runtime-trap.json"
jq empty "$root/examples/triad-packet.waw-doc-owner-trap.json"
jq empty "$root/examples/triad-packet.dataset-noise.json"
jq empty "$root/examples/triad-packet.negative-shortcut-base.json"
jq empty "$root/examples/triad-packet.negative-shortcut-lanes.json"
jq empty "$root/examples/triad-packet.pack6m-replay-waw.json"
jq empty "$root/examples/triad-packet.source-weighting.json"
jq empty "$root/examples/triad-packet.auto-query-memory.json"
jq empty "$root/examples/triad-packet.polarization-role-swap.json"
jq empty "$root/examples/triad-packet.polarization-reversed-stop.json"
jq empty "$root/examples/triad-packet.route-balanced-focus.json"
jq empty "$root/examples/triad-packet.hgate-size-only.json"
jq empty "$root/examples/triad-packet.canonical-alias-pass.json"
jq empty "$root/examples/triad-packet.canonical-alias-veto.json"
jq empty "$root/examples/triad-packet.canonical-alias-conflict.json"
jq empty "$root/examples/eval-corpus.json"
jq empty "$root/examples/probe-corpus.json"
jq empty "$root/examples/waw-corpus.json"

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
grep -q '"core_version": "sparse-triad-v3.3-modular-router"' <<<"$map_json"
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

hgate_json="$("$root/nanda-structural-gate/scripts/nanda-hgate" "$root/examples/triad-packet.hgate-size-only.json" --input-format json)"
jq -e '.mode == "hierarchical-gate"' <<<"$hgate_json" >/dev/null
jq -e '.hierarchical_decision.action == "STRUCTURALLY_ACCEPTED"' <<<"$hgate_json" >/dev/null
jq -e '.hierarchical_decision.global_verdict == "WATCH"' <<<"$hgate_json" >/dev/null
jq -e '.hierarchical_decision.global_size_only == true' <<<"$hgate_json" >/dev/null
jq -e '.hierarchical_decision.local_pass == 17 and .hierarchical_decision.branches == 17' <<<"$hgate_json" >/dev/null
set +e
hgate_splice_json="$("$root/nanda-structural-gate/scripts/nanda-hgate" "$root/examples/triads.code-flow-splice.md" --domain code --format json)"
hgate_splice_status=$?
set -e
if [[ "$hgate_splice_status" -ne 1 ]]; then
  echo "expected hgate splice to return VETO status" >&2
  echo "$hgate_splice_json" >&2
  exit 1
fi
jq -e '.hierarchical_decision.action == "REPAIR_REQUIRED"' <<<"$hgate_splice_json" >/dev/null
jq -e '.hierarchical_decision.global_foreign_pull > 0' <<<"$hgate_splice_json" >/dev/null

search_json="$("$search" "$root/examples/triad-packet.interference-search.json" --input-format json --top-k 3)"
grep -q '"mode": "interference-retrieval"' <<<"$search_json"

tmp_focus_packet="$(mktemp)"
focus_json="$("$focus" "$root/examples/triad-packet.route-balanced-focus.json" --input-format json --max-triads 12 --route-cap 4 --route-triad-cap 4 --out "$tmp_focus_packet")"
jq -e '.mode == "focused-packet-builder"' <<<"$focus_json" >/dev/null
jq -e '.focused_memory_size <= 12' <<<"$focus_json" >/dev/null
jq -e '.runtime_contract.state == "PACKED_RUNTIME_READY"' <<<"$focus_json" >/dev/null
jq empty "$tmp_focus_packet"
"$budget" "$tmp_focus_packet" --input-format json >/dev/null
"$search" "$tmp_focus_packet" --input-format json --top-k 2 >/dev/null
rm -f "$tmp_focus_packet"
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
source_weight_json="$("$search" "$root/examples/triad-packet.source-weighting.json" --input-format json --top-k 3)"
jq -e '.peaks[0].peak == "current-frontier"' <<<"$source_weight_json" >/dev/null
jq -e '.source_weighting.enabled == true' <<<"$source_weight_json" >/dev/null
jq -e '[.peaks[0].supporting_triads[].source_weight] | min >= 1.0' <<<"$source_weight_json" >/dev/null
auto_query_json="$("$search" "$root/examples/triad-packet.auto-query-memory.json" --input-format json --top-k 3)"
jq -e '.query.source == "auto_query_triads"' <<<"$auto_query_json" >/dev/null
jq -e '.query.triads | length > 0' <<<"$auto_query_json" >/dev/null
jq -e '.peaks[0].peak == "lower-operator-debt-route"' <<<"$auto_query_json" >/dev/null
auto_query_override_json="$("$search" "$root/examples/triad-packet.auto-query-memory.json" --input-format json --query "lower operator debt route" --top-k 3)"
jq -e '.query.text == "lower operator debt route"' <<<"$auto_query_override_json" >/dev/null
jq -e '.query.source == "auto_query_triads"' <<<"$auto_query_override_json" >/dev/null
jq -e '.peaks[0].peak == "lower-operator-debt-route"' <<<"$auto_query_override_json" >/dev/null
polarization_json="$("$search" "$root/examples/triad-packet.polarization-role-swap.json" --input-format json --top-k 3)"
jq -e '.peaks[0].peak == "payment-forward"' <<<"$polarization_json" >/dev/null
jq -e '.peaks[0].polarization.state == "ALIGNED"' <<<"$polarization_json" >/dev/null
jq -e '.peaks[0].supporting_triads[0].polarity == "payer->payment->document"' <<<"$polarization_json" >/dev/null
jq -e '.coarse_to_fine.state == "LOCALIZED"' <<<"$polarization_json" >/dev/null
polarity_stop_json="$("$search" "$root/examples/triad-packet.polarization-reversed-stop.json" --input-format json --top-k 3)"
jq -e '.peaks[0].peak == "payment-reversed"' <<<"$polarity_stop_json" >/dev/null
jq -e '.peaks[0].polarization.state == "REVERSED"' <<<"$polarity_stop_json" >/dev/null
jq -e '.peaks[0].polarization_penalty == 0.18' <<<"$polarity_stop_json" >/dev/null
jq -e '.peak_decision.state == "POLARITY_REVERSED" and .peak_decision.safe_to_answer == false' <<<"$polarity_stop_json" >/dev/null
jq -e '.field_interpretation.state == "polarity_reversed"' <<<"$polarity_stop_json" >/dev/null
jq -e '.field_state_machine.state == "FIELD_REVERSED" and .field_state_machine.safe_to_answer == false' <<<"$polarity_stop_json" >/dev/null
jq -e '.field_state_machine.action == "STOP_REPAIR_POLARITY"' <<<"$polarity_stop_json" >/dev/null
balanced_json="$("$search" "$root/examples/triad-packet.route-balanced-focus.json" --input-format json --query "lower operator debt route" --route-cap 3 --route-triad-cap 1 --top-k 3)"
jq -e '.route_balanced_focus.enabled == true' <<<"$balanced_json" >/dev/null
jq -e '.route_balanced_focus.original_memory_size == 6 and .route_balanced_focus.focused_memory_size == 2' <<<"$balanced_json" >/dev/null
jq -e '.peaks[0].peak == "lower-operator-debt-route"' <<<"$balanced_json" >/dev/null
jq -e '.coarse_to_fine.state == "LOCALIZED"' <<<"$balanced_json" >/dev/null
jq -e '.field_state_machine.state == "FIELD_CONTESTED" and .field_state_machine.safe_to_answer == false' <<<"$balanced_json" >/dev/null
jq -e '.field_state_machine.signals.route_balanced == true' <<<"$balanced_json" >/dev/null
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
jq -e '.field_interpretation.state == "contested"' <<<"$noisy_search_json" >/dev/null
jq -e '.peak_decision.state == "WATCH"' <<<"$noisy_search_json" >/dev/null
jq -e '.peak_decision.safe_to_answer == false' <<<"$noisy_search_json" >/dev/null
jq -e '.field_state_machine.state == "FIELD_CONTESTED" and .field_state_machine.safe_to_answer == false' <<<"$noisy_search_json" >/dev/null
jq -e '.field_state_machine.action == "SPLIT_OR_QUERY"' <<<"$noisy_search_json" >/dev/null
eval_json="$("$evaler" \
  --case "$root/examples/triad-packet.interference-search-route-trap.json:certification:FOCUSED" \
  --case "$root/examples/triad-packet.interference-search-noisy.json:certification:WATCH")"
jq -e '.mode == "eval-suite"' <<<"$eval_json" >/dev/null
jq -e '.passed == 2 and .total == 2 and .accuracy == 1' <<<"$eval_json" >/dev/null
eval_suite_json="$("$evaler" --suite "$root/examples/eval-corpus.json")"
jq -e '.mode == "eval-suite"' <<<"$eval_suite_json" >/dev/null
jq -e '.passed == 2 and .total == 2 and .accuracy == 1' <<<"$eval_suite_json" >/dev/null
waw_json="$("$waw" --suite "$root/examples/waw-corpus.json")"
jq -e '.mode == "waw-benchmark"' <<<"$waw_json" >/dev/null
jq -e '.passed == 3 and .total == 3 and .waw_score == 1' <<<"$waw_json" >/dev/null
jq -e '.structural_wins == 3 and .lexical_traps == 3 and .explainable_drifts == 3' <<<"$waw_json" >/dev/null
set +e
dataset_json="$("$dataset_doctor" "$root/examples/triad-packet.dataset-noise.json" --input-format json --route-cap 8)"
dataset_status=$?
set -e
if [[ "$dataset_status" -ne 3 ]]; then
  echo "expected dataset-doctor WATCH status" >&2
  echo "$dataset_json" >&2
  exit 1
fi
jq -e '.mode == "dataset-doctor" and .verdict == "WATCH"' <<<"$dataset_json" >/dev/null
jq -e '([.warnings[].kind] | index("large_unbalanced_corpus") and index("route_imbalance") and index("hub_dominance") and index("duplicate_current") and index("weak_text_query"))' <<<"$dataset_json" >/dev/null
budget_json="$("$budget" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json)"
jq -e '.mode == "nanda-6m-budget-planner"' <<<"$budget_json" >/dev/null
jq -e '.state == "FITS_L3"' <<<"$budget_json" >/dev/null
jq -e '.safe_for_hot_core == true' <<<"$budget_json" >/dev/null
jq -e '.runtime_focus.state == "PACKED_RUNTIME_READY" and .runtime_focus.focus_triads_capacity == 15000 and .runtime_focus.default_focus_field_requests == 64' <<<"$budget_json" >/dev/null
jq -e '.hard_budget_bytes == 6291456' <<<"$budget_json" >/dev/null
jq -e '.capacity.triads == 65536' <<<"$budget_json" >/dev/null
pack6m_json="$("$pack6m" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json)"
jq -e '.mode == "nanda-6m-pack-skeleton"' <<<"$pack6m_json" >/dev/null
jq -e '.state == "PACKED_FITS_L3" and .packed_ok == true' <<<"$pack6m_json" >/dev/null
jq -e '.packed_records.count == 10 and .packed_records.memory_count == 8 and .packed_records.query_count == 2 and .packed_records.record_bytes == 32' <<<"$pack6m_json" >/dev/null
jq -e '.packed_records.sample[0].wave_seed > 0 and .packed_records.sample[0].check > 0' <<<"$pack6m_json" >/dev/null
jq -e '.dictionaries.entities.fits == true and .dictionaries.roles.fits == true' <<<"$pack6m_json" >/dev/null
jq -e '.projection.source == "candidate_triads" and .projection.records == 2 and .projection.wave_dim == 1024 and .projection.bytes == 2048' <<<"$pack6m_json" >/dev/null
jq -e '.projection.summary.nonzero > 0 and .projection.summary.energy > 0' <<<"$pack6m_json" >/dev/null
jq -e '.projection.sample | length == 8' <<<"$pack6m_json" >/dev/null
jq -e '.centroids.source == "memory_triads" and .centroids.record_bytes == 1024 and .centroids.route_count == 3 and .centroids.group_count == 3' <<<"$pack6m_json" >/dev/null
jq -e '.centroids.total_count == 6 and .centroids.route[0].summary.energy > 0' <<<"$pack6m_json" >/dev/null
jq -e '.centroids.route[0].score.cosine > 0' <<<"$pack6m_json" >/dev/null
jq -e '.peaks.mode == "packed-candidate-query-vs-memory-centroid-cosine"' <<<"$pack6m_json" >/dev/null
jq -e '.peaks.route.state == "PEAK_THIN" and .peaks.route.top_score > 0 and .peaks.route.margin >= 0' <<<"$pack6m_json" >/dev/null
jq -e '.peaks.group.state == "PEAK_THIN" and .peaks.group.top_score > 0 and .peaks.group.margin >= 0' <<<"$pack6m_json" >/dev/null
jq -e '.peak_decision.state == "PACKED_THIN" and .peak_decision.verdict == "WATCH" and .peak_decision.safe_to_answer == false' <<<"$pack6m_json" >/dev/null
jq -e '.peak_decision.route.top_id > 0 and .peak_decision.group.top_id > 0 and .peak_decision.thresholds.min_focus_score == 0.01' <<<"$pack6m_json" >/dev/null
jq -e '.packed_support.mode == "query-vs-memory-triad-contributors"' <<<"$pack6m_json" >/dev/null
jq -e '.packed_support.route.top_id == .peaks.route.top_id and .packed_support.route.considered == 3' <<<"$pack6m_json" >/dev/null
jq -e '.packed_support.route.support_count == 1 and .packed_support.route.anti_count == 2 and .packed_support.route.net_dot == 32' <<<"$pack6m_json" >/dev/null
jq -e '.packed_support.route.support[0].dot > 0 and .packed_support.route.anti[0].dot < 0' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_keys.mode == "stable-lane-keys" and .packed_lane_keys.storage == "cold-stable-signature"' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_keys.route.key_hash == 2855017131 and .packed_lane_keys.route.anti_count == 2' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_keys.route.compile_hint.record_mask_a == 96 and .packed_lane_keys.route.compile_hint.protected_support_mask_a == 16' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lanes.mode == "packed-lane-preview" and .packed_lanes.lane_bytes == 64' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lanes.route.key_hash == .packed_lane_keys.route.key_hash and .packed_lanes.route.compiled_storage == "hot-packed-lane64"' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lanes.route.state == "LANE_PREVIEW_READY" and .packed_lanes.route.action == "suppress_anti_support"' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lanes.route.before_net_dot == 32 and .packed_lanes.route.after_net_dot == 288 and .packed_lanes.route.delta_dot == 256' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lanes.route.record_mask_a == 96 and .packed_lanes.route.protected_support_mask_a == 16' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_store.mode == "packed-lane-store" and .packed_lane_store.storage == "hot-compiled-lane-arena"' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_store.capacity == 16384 and .packed_lane_store.count == 2 and .packed_lane_store.bytes == 128' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_store.sample[0].key_hash == .packed_lanes.route.key_hash and .packed_lane_store.sample[0].record_mask_a == 96' <<<"$pack6m_json" >/dev/null
jq -e '.runtime_contract.mode == "packed-hot-runtime-contract" and .runtime_contract.state == "PACKED_RUNTIME_READY" and .runtime_contract.ready == true' <<<"$pack6m_json" >/dev/null
jq -e '.runtime_contract.focus_triads_capacity == 15000 and .runtime_contract.focus_window_fits == true and .runtime_contract.default_focus_field_requests == 64' <<<"$pack6m_json" >/dev/null
jq -e '.runtime_contract.workspace_model.score_arrays == 3 and .runtime_contract.workspace_model.score_bytes == 16 and .runtime_contract.workspace_model.support_field_bytes == 56' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_replay.mode == "feedback-lane-replay" and .packed_lane_replay.state == "PACKED_LANE_REPLAY_NONE" and .packed_lane_replay.matched_keys == 0' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_replay.touch_policy.mode == "observer-to-compute-sweep" and .packed_lane_replay.stability_state == "NO_REPLAY_FIELD"' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_replay.stability_sweep[0].label == "observer" and .packed_lane_replay.stability_sweep[3].label == "full_touch"' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_replay.computational_effect.state == "REPLAY_COMPUTE_NONE" and .packed_lane_replay.computational_effect.safe_to_answer == false' <<<"$pack6m_json" >/dev/null
jq -e '.packed_replay_decision.mode == "replay-adjusted-peak-firewall" and .packed_replay_decision.stability_verdict == "NO_REPLAY_EVIDENCE"' <<<"$pack6m_json" >/dev/null
jq -e '.packed_replay_decision.core == "nanda_6m::evaluate_replay" and .packed_replay_decision.hot_compatible == true' <<<"$pack6m_json" >/dev/null
jq -e '.packed_replay_decision.firewall.blocks_direct_pass == true and .packed_replay_decision.safe_to_answer == false' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_application.mode == "single-pass-suppress-anti-support"' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_application.raw_state == "PACKED_THIN" and .packed_lane_application.state == "PACKED_LANE_FOCUSED_CANDIDATE"' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_application.ready_for_hot_loop == true and .packed_lane_application.safe_to_answer == false' <<<"$pack6m_json" >/dev/null
jq -e '.packed_lane_application.route.state == "LANE_AXIS_FOCUSED_CANDIDATE" and .packed_lane_application.route.after_net_dot == 288' <<<"$pack6m_json" >/dev/null
pack6m_replay_json="$("$pack6m" "$root/examples/triad-packet.negative-shortcut-lanes.json" --input-format json)"
jq -e '.packed_lane_replay.state == "PACKED_LANE_REPLAY_FOCUSED" and .packed_lane_replay.matched_keys == 2 and .packed_lane_replay.compiled_lanes == 2' <<<"$pack6m_replay_json" >/dev/null
jq -e '.packed_lane_replay.sample[0].source == "negative_shortcuts" and .packed_lane_replay.sample[0].query_match_ratio == 1' <<<"$pack6m_replay_json" >/dev/null
jq -e '.packed_lane_replay.before_net_dot == 2816 and .packed_lane_replay.after_net_dot == 3456 and .packed_lane_replay.delta_dot == 640' <<<"$pack6m_replay_json" >/dev/null
jq -e '.packed_lane_replay.stability_state == "STABLE_UNDER_SOFT_TOUCH"' <<<"$pack6m_replay_json" >/dev/null
jq -e '.packed_lane_replay.stability_sweep[1].label == "soft_touch" and .packed_lane_replay.stability_sweep[1].after_net_dot == 2976 and .packed_lane_replay.stability_sweep[1].field_state == "FIELD_FOCUSED_BY_REPLAY"' <<<"$pack6m_replay_json" >/dev/null
jq -e '.packed_lane_replay.computational_effect.state == "REPLAY_COMPUTE_READY" and .packed_lane_replay.computational_effect.field_after == 3456 and .packed_lane_replay.computational_effect.safe_to_answer == false' <<<"$pack6m_replay_json" >/dev/null
jq -e '.packed_replay_decision.stability_verdict == "STABLE_WITH_REPLAY" and .packed_replay_decision.verdict == "PASS" and .packed_replay_decision.safe_to_answer == false' <<<"$pack6m_replay_json" >/dev/null
pack6m_waw_json="$("$pack6m" "$root/examples/triad-packet.pack6m-replay-waw.json" --input-format json)"
jq -e '.peak_decision.state == "PACKED_THIN" and .peak_decision.safe_to_answer == false' <<<"$pack6m_waw_json" >/dev/null
jq -e '.packed_lane_replay.stability_sweep[1].after_net_dot == 192 and .packed_lane_replay.stability_sweep[1].field_state == "FIELD_FOCUSED_BY_REPLAY"' <<<"$pack6m_waw_json" >/dev/null
jq -e '.packed_replay_decision.stability_verdict == "REPLAY_RESCUED_THIN_FIELD" and .packed_replay_decision.action == "REVIEW_REPLAY_RESCUED_FIELD"' <<<"$pack6m_waw_json" >/dev/null
jq -e '.packed_replay_decision.verdict == "WATCH" and .packed_replay_decision.safe_to_answer == false and .packed_replay_decision.firewall.requires_structural_gate == true' <<<"$pack6m_waw_json" >/dev/null
doctor_json="$("$doctor")"
jq -e '.mode == "doctor" and .healthy == true' <<<"$doctor_json" >/dev/null
jq -e '.route_trap.top == "certification" and .route_trap.state == "FOCUSED"' <<<"$doctor_json" >/dev/null
jq -e '.route_trap.field_state == "FIELD_FOCUSED" and .route_trap.field_safe_to_answer == true' <<<"$doctor_json" >/dev/null
jq -e '.noisy.state == "WATCH" and .noisy.safe_to_answer == false' <<<"$doctor_json" >/dev/null
jq -e '.noisy.field_state == "FIELD_CONTESTED" and .noisy.field_safe_to_answer == false' <<<"$doctor_json" >/dev/null
trap_search_json="$("$search" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --top-k 3)"
jq -e '.verdict == "PASS" and .field_state == "FIELD_FOCUSED" and .safe_to_answer == true and .top_peak == "certification"' <<<"$trap_search_json" >/dev/null
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
jq -e '.field_state_machine.state == "FIELD_FOCUSED" and .field_state_machine.safe_to_answer == true' <<<"$trap_search_json" >/dev/null
jq -e '.field_state_machine.signals.lexical_trap_detected == true' <<<"$trap_search_json" >/dev/null
jq -e '.field_interpretation.lexical_trap_detected == true' <<<"$trap_search_json" >/dev/null
jq -e '.field_interpretation.centroid_drift.route.changed == true' <<<"$trap_search_json" >/dev/null
serve_json="$(printf '{"command":"doctor"}\n' | "$serve")"
jq -e '.ok == true and .result.mode == "doctor" and .result.healthy == true' <<<"$serve_json" >/dev/null
tmp_search="$(mktemp)"
tmp_feedback="$(mktemp)"
printf '%s\n' "$trap_search_json" >"$tmp_search"
"$feedback" "$tmp_search" --decision accept --note "route trap accepted" --out "$tmp_feedback" >/dev/null
jq empty "$tmp_feedback"
jq -e '.mode == "feedback-memory"' "$tmp_feedback" >/dev/null
jq -e '.decision == "accept"' "$tmp_feedback" >/dev/null
jq -e '.peak == "certification"' "$tmp_feedback" >/dev/null
jq -e '.memory_patch.reinforce_peak == "certification"' "$tmp_feedback" >/dev/null
jq -e '.positive_shortcuts[0].reinforce_peak == "certification"' "$tmp_feedback" >/dev/null
tmp_positive_index="$(mktemp)"
"$indexer" "$root/examples/triad-packet.interference-search-route-trap.json" "$tmp_feedback" --input-format json --out "$tmp_positive_index" >/dev/null
jq -e '.positive_shortcuts[0].accepted_count == 1' "$tmp_positive_index" >/dev/null
positive_search_json="$("$search" "$tmp_positive_index" --input-format json --query-file "$root/examples/triad-packet.interference-search-route-trap.json" --query-format json --top-k 3)"
jq -e '.constructive_interference.applied == true' <<<"$positive_search_json" >/dev/null
jq -e '.constructive_interference.reinforcements[0].reinforce_peak == "certification"' <<<"$positive_search_json" >/dev/null
jq -e '.peaks[0].positive_lane_boost > 0' <<<"$positive_search_json" >/dev/null
tmp_feedback2="$(mktemp)"
tmp_positive_learned_index="$(mktemp)"
"$feedback" "$tmp_search" --decision accept --note "route trap accepted" --out "$tmp_feedback2" >/dev/null
"$indexer" "$root/examples/triad-packet.interference-search-route-trap.json" "$tmp_feedback" "$tmp_feedback2" --input-format json --out "$tmp_positive_learned_index" >/dev/null
jq -e '.positive_shortcuts[0].accepted_count == 2' "$tmp_positive_learned_index" >/dev/null
learned_positive_json="$("$search" "$tmp_positive_learned_index" --input-format json --query-file "$root/examples/triad-packet.interference-search-route-trap.json" --query-format json --top-k 3)"
jq -e '.constructive_interference.reinforcements[0].effective_boost > 0.08' <<<"$learned_positive_json" >/dev/null
jq -e '.constructive_interference.reinforcements[0].accepted_count == 2' <<<"$learned_positive_json" >/dev/null
rm -f "$tmp_search" "$tmp_feedback" "$tmp_positive_index"
rm -f "$tmp_feedback2" "$tmp_positive_learned_index"
tmp_index="$(mktemp)"
"$indexer" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --out "$tmp_index" >/dev/null
jq empty "$tmp_index"
indexed_search_json="$("$search" "$tmp_index" --input-format json --query-file "$root/examples/triad-packet.interference-search-route-trap.json" --query-format json --top-k 3)"
jq -e '.peak_decision.state == "FOCUSED"' <<<"$indexed_search_json" >/dev/null
jq -e '.wins_over_lexical_baseline == true' <<<"$indexed_search_json" >/dev/null
rm -f "$tmp_index"
negative_base_json="$("$search" "$root/examples/triad-packet.negative-shortcut-base.json" --input-format json --top-k 3)"
jq -e '.peaks[0].peak == "customs"' <<<"$negative_base_json" >/dev/null
jq -e '.destructive_interference.applied == false' <<<"$negative_base_json" >/dev/null
negative_lanes_json="$("$search" "$root/examples/triad-packet.negative-shortcut-lanes.json" --input-format json --top-k 3)"
jq -e '.verdict == "WATCH" and .field_state == "FIELD_THIN" and .safe_to_answer == false and .top_peak == "certification"' <<<"$negative_lanes_json" >/dev/null
jq -e '.peaks[0].peak == "certification"' <<<"$negative_lanes_json" >/dev/null
jq -e '.destructive_interference.applied == true' <<<"$negative_lanes_json" >/dev/null
jq -e '.destructive_interference.suppressions[0].suppressed_peak == "customs"' <<<"$negative_lanes_json" >/dev/null
jq -e '.destructive_interference.suppressions[0].suppress_peak == "customs"' <<<"$negative_lanes_json" >/dev/null
negative_lanes_group_json="$("$search" "$root/examples/triad-packet.negative-shortcut-lanes.json" --input-format json --group-by group --top-k 5)"
jq -e '.top_peak == "certification-route"' <<<"$negative_lanes_group_json" >/dev/null
jq -e '.destructive_interference.applied == true' <<<"$negative_lanes_group_json" >/dev/null
jq -e '.destructive_interference.suppressions[0].suppressed_peak == "customs-shortcut"' <<<"$negative_lanes_group_json" >/dev/null
jq -e '.destructive_interference.suppressions[0].suppress_peak == "customs-shortcut"' <<<"$negative_lanes_group_json" >/dev/null
probe_json="$("$probe" "$root/examples/triad-packet.negative-shortcut-lanes.json" --input-format json --top-k 3)"
jq -e '.mode == "probe-report" and .decision == "SHIFTED_TO_REVIEW"' <<<"$probe_json" >/dev/null
jq -e '.plain.top_peak == "customs" and .negative.top_peak == "certification"' <<<"$probe_json" >/dev/null
jq -e '.delta.top_changed == true and .delta.suppression_count == 1' <<<"$probe_json" >/dev/null
probe_external_json="$("$probe" "$root/examples/triad-packet.negative-shortcut-base.json" --input-format json --negative "$root/examples/triad-packet.negative-shortcut-lanes.json" --top-k 3)"
jq -e '.decision == "SHIFTED_TO_REVIEW" and .plain.top_peak == "customs" and .negative.top_peak == "certification"' <<<"$probe_external_json" >/dev/null
probe_suite_json="$("$probe" --suite "$root/examples/probe-corpus.json" --input-format json --top-k 3)"
jq -e '.mode == "probe-suite" and .passed == 4 and .total == 4 and .accuracy == 1' <<<"$probe_suite_json" >/dev/null
positive_fixture_json="$("$search" "$root/examples/triad-packet.positive-lanes.json" --input-format json --top-k 3)"
jq -e '.top_peak == "certification"' <<<"$positive_fixture_json" >/dev/null
jq -e '.constructive_interference.applied == true' <<<"$positive_fixture_json" >/dev/null
jq -e '.constructive_interference.reinforcements[0].reinforce_peak == "certification"' <<<"$positive_fixture_json" >/dev/null
jq -e '.peaks[0].positive_lane_boost > 0' <<<"$positive_fixture_json" >/dev/null
tmp_negative_search="$(mktemp)"
tmp_negative_feedback="$(mktemp)"
tmp_negative_index="$(mktemp)"
printf '%s\n' "$negative_base_json" >"$tmp_negative_search"
"$feedback" "$tmp_negative_search" --decision reject --note "customs shortcut" --out "$tmp_negative_feedback" >/dev/null
jq -e '.negative_shortcuts[0].suppress_peak == "customs"' "$tmp_negative_feedback" >/dev/null
jq -e '.negative_shortcuts[0].suppress_route == "customs"' "$tmp_negative_feedback" >/dev/null
jq -e '.negative_shortcuts[0].suppress_group == "customs-shortcut"' "$tmp_negative_feedback" >/dev/null
jq -e '.negative_shortcuts[0].prefer_peak == "certification"' "$tmp_negative_feedback" >/dev/null
jq -e '.negative_shortcuts[0].support_terms | length > 0' "$tmp_negative_feedback" >/dev/null
"$indexer" "$root/examples/triad-packet.negative-shortcut-base.json" "$tmp_negative_feedback" --input-format json --out "$tmp_negative_index" >/dev/null
indexed_negative_json="$("$search" "$tmp_negative_index" --input-format json --query-file "$root/examples/triad-packet.negative-shortcut-base.json" --query-format json --top-k 3)"
jq -e '.peaks[0].peak == "certification"' <<<"$indexed_negative_json" >/dev/null
jq -e '.destructive_interference.applied == true' <<<"$indexed_negative_json" >/dev/null
tmp_negative_feedback2="$(mktemp)"
tmp_negative_learned_index="$(mktemp)"
"$feedback" "$tmp_negative_search" --decision reject --note "customs shortcut" --out "$tmp_negative_feedback2" >/dev/null
"$indexer" "$root/examples/triad-packet.negative-shortcut-base.json" "$tmp_negative_feedback" "$tmp_negative_feedback2" --input-format json --out "$tmp_negative_learned_index" >/dev/null
jq -e '.negative_shortcuts[0].rejected_count == 2' "$tmp_negative_learned_index" >/dev/null
learned_negative_json="$("$search" "$tmp_negative_learned_index" --input-format json --query-file "$root/examples/triad-packet.negative-shortcut-base.json" --query-format json --top-k 3)"
jq -e '.destructive_interference.suppressions[0].effective_penalty > 0.18' <<<"$learned_negative_json" >/dev/null
jq -e '.destructive_interference.suppressions[0].rejected_count == 2' <<<"$learned_negative_json" >/dev/null
rm -f "$tmp_negative_search" "$tmp_negative_feedback" "$tmp_negative_index"
rm -f "$tmp_negative_feedback2" "$tmp_negative_learned_index"
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
grep -q '"local_branches": 14' <<<"$dogfood_json"
grep -q '"local_pass": 14' <<<"$dogfood_json"
dogfood_text="$("$dogfood" "$root")"
grep -q 'ACTION: SAFE_TO_EDIT' <<<"$dogfood_text"
grep -q 'BRANCHES: 14/14 PASS' <<<"$dogfood_text"
dogfood_refactor_json="$("$dogfood" "$root" --refactor-plan --format json)"
jq -e '.refactor_plan.mode == "code-map"' <<<"$dogfood_refactor_json" >/dev/null
jq -e '.refactor_plan.clusters | length > 0' <<<"$dogfood_refactor_json" >/dev/null
code_map_json="$("$code_mapper" "$root/src/main.rs" --format json)"
jq -e '.mode == "code-map"' <<<"$code_map_json" >/dev/null
jq -e '.clusters | length > 0' <<<"$code_map_json" >/dev/null
jq -e '.clusters[] | select(.cluster == "cli-router")' <<<"$code_map_json" >/dev/null

"$init_md" --task-id skill-smoke --template skill --stdout >/dev/null
"$init_md" --task-id project-smoke --template project --stdout >/dev/null
"$doctor" --help | grep -q "Usage: nanda doctor"
"$evaler" --help | grep -q "Usage: nanda eval"
"$waw" --help | grep -q "Usage: nanda waw"
"$dataset_doctor" --help | grep -q "Usage: nanda dataset-doctor"
"$aliases" --help | grep -q "Usage: nanda aliases"
alias_json="$("$aliases" "$root/examples/triad-packet.canonical-alias-pass.json" --input-format json)"
jq -e '.canonicalization.enabled == true and .canonicalization.applied_count == 1' <<<"$alias_json" >/dev/null
alias_pass="$("$checker" --triads "$root/examples/triad-packet.canonical-alias-pass.json" --format json)"
jq -e '.verdict == "PASS" and .canonicalization.applied_count == 1' <<<"$alias_pass" >/dev/null
set +e
alias_veto="$("$checker" --triads "$root/examples/triad-packet.canonical-alias-veto.json" --format json)"
alias_veto_status=$?
set -e
if [[ "$alias_veto_status" -ne 1 ]]; then
  echo "expected canonical alias issuer conflict to VETO" >&2
  echo "$alias_veto" >&2
  exit 1
fi
jq -e '.verdict == "VETO" and (.conflicts | length) > 0 and .canonicalization.applied_count == 1' <<<"$alias_veto" >/dev/null
set +e
alias_conflict="$("$aliases" "$root/examples/triad-packet.canonical-alias-conflict.json" --input-format json)"
alias_conflict_status=$?
set -e
if [[ "$alias_conflict_status" -ne 3 ]]; then
  echo "expected ambiguous aliases to WATCH" >&2
  echo "$alias_conflict" >&2
  exit 1
fi
jq -e '.canonicalization.conflict_count == 1' <<<"$alias_conflict" >/dev/null
"$budget" --help | grep -q "Usage: nanda budget"
"$pack6m" --help | grep -q "Usage: nanda pack6m"
"$bench6m" --help | grep -q "Usage: nanda bench6m"
bench6m_json="$("$bench6m" --replay-iterations 1000 --projection-iterations 10 --triads 8 --format json)"
jq -e '.mode == "nanda-6m-hot-benchmark"' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.replay.iterations == 1000 and .benchmarks.replay.ns_per_op > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.projection.iterations == 10 and .benchmarks.projection.triads_in_window == 8 and .benchmarks.projection.ns_per_op > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.lane_application.iterations == 1000000 and .benchmarks.lane_application.kernel == "compile_and_apply_suppress_anti_lane" and .benchmarks.lane_application.ns_per_op > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.lane_sweep.iterations == 100000 and .benchmarks.lane_sweep.kernel == "apply_suppress_anti_lane_sweep" and .benchmarks.lane_sweep.fields == 64 and .benchmarks.lane_sweep.ns_per_field > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.aligned_lane_sweep.iterations == 100000 and .benchmarks.aligned_lane_sweep.kernel == "apply_aligned_suppress_anti_lane_sweep" and .benchmarks.aligned_lane_sweep.fields == 64 and .benchmarks.aligned_lane_sweep.ns_per_field > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.aligned_compile_sweep.iterations == 100000 and .benchmarks.aligned_compile_sweep.kernel == "compile_and_apply_aligned_suppress_anti_lane_sweep" and .benchmarks.aligned_compile_sweep.fields == 64 and .benchmarks.aligned_compile_sweep.ns_per_field > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.support_build.iterations == 100 and .benchmarks.support_build.kernel == "build_support_fields" and .benchmarks.support_build.fields == 64 and .benchmarks.support_build.ns_per_field > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.support_build_compile_sweep.iterations == 100 and .benchmarks.support_build_compile_sweep.kernel == "build_support_fields_and_compile_sweep" and .benchmarks.support_build_compile_sweep.fields == 64 and .benchmarks.support_build_compile_sweep.ns_per_field > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.support_score_build.iterations == 100 and .benchmarks.support_score_build.kernel == "build_support_scores_and_fields" and .benchmarks.support_score_build.fields == 64 and .benchmarks.support_score_build.ns_per_field > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.support_score_build_compile_sweep.iterations == 100 and .benchmarks.support_score_build_compile_sweep.kernel == "build_support_scores_fields_and_compile_sweep" and .benchmarks.support_score_build_compile_sweep.fields == 64 and .benchmarks.support_score_build_compile_sweep.ns_per_field > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.support_bucket_build.iterations == 100 and .benchmarks.support_bucket_build.kernel == "build_support_score_buckets_and_fields" and .benchmarks.support_bucket_build.fields == 64 and .benchmarks.support_bucket_build.ns_per_field > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.support_bucket_build_compile_sweep.iterations == 100 and .benchmarks.support_bucket_build_compile_sweep.kernel == "build_support_score_buckets_fields_and_compile_sweep" and .benchmarks.support_bucket_build_compile_sweep.fields == 64 and .benchmarks.support_bucket_build_compile_sweep.ns_per_field > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.hot_cycle.iterations == 100 and .benchmarks.hot_cycle.kernel == "run_packed_hot_cycle" and .benchmarks.hot_cycle.fields == 64 and .benchmarks.hot_cycle.ns_per_field > 0' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.hot_cycle.runtime_contract.state == "PACKED_RUNTIME_READY" and .benchmarks.hot_cycle.runtime_contract.ready == true and .benchmarks.hot_cycle.runtime_contract.workspace_fits == true' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.hot_cycle.runtime_contract.focus_triads_capacity == 15000 and .benchmarks.hot_cycle.runtime_contract.default_focus_field_requests == 64' <<<"$bench6m_json" >/dev/null
bench6m_lane_json="$("$bench6m" --mode lane --lane-iterations 1000 --format json)"
jq -e '.benchmarks.replay == null and .benchmarks.projection == null' <<<"$bench6m_lane_json" >/dev/null
jq -e '.benchmarks.lane_application.iterations == 1000 and .benchmarks.lane_application.ops_per_second > 0' <<<"$bench6m_lane_json" >/dev/null
bench6m_lane_sweep_json="$("$bench6m" --mode lane-sweep --lane-sweep-iterations 1000 --lane-sweep-width 8 --format json)"
jq -e '.benchmarks.replay == null and .benchmarks.projection == null and .benchmarks.lane_application == null and .benchmarks.aligned_lane_sweep == null' <<<"$bench6m_lane_sweep_json" >/dev/null
jq -e '.benchmarks.lane_sweep.iterations == 1000 and .benchmarks.lane_sweep.fields == 8 and .benchmarks.lane_sweep.compiled_lanes == 8 and .benchmarks.lane_sweep.ops_per_second > 0' <<<"$bench6m_lane_sweep_json" >/dev/null
bench6m_aligned_lane_sweep_json="$("$bench6m" --mode aligned-lane-sweep --lane-sweep-iterations 1000 --lane-sweep-width 8 --format json)"
jq -e '.benchmarks.replay == null and .benchmarks.projection == null and .benchmarks.lane_application == null and .benchmarks.lane_sweep == null and .benchmarks.aligned_compile_sweep == null' <<<"$bench6m_aligned_lane_sweep_json" >/dev/null
jq -e '.benchmarks.aligned_lane_sweep.iterations == 1000 and .benchmarks.aligned_lane_sweep.fields == 8 and .benchmarks.aligned_lane_sweep.compiled_lanes == 8 and .benchmarks.aligned_lane_sweep.ops_per_second > 0' <<<"$bench6m_aligned_lane_sweep_json" >/dev/null
bench6m_aligned_compile_sweep_json="$("$bench6m" --mode aligned-compile-sweep --lane-sweep-iterations 1000 --lane-sweep-width 8 --format json)"
jq -e '.benchmarks.replay == null and .benchmarks.projection == null and .benchmarks.lane_application == null and .benchmarks.lane_sweep == null and .benchmarks.aligned_lane_sweep == null' <<<"$bench6m_aligned_compile_sweep_json" >/dev/null
jq -e '.benchmarks.aligned_compile_sweep.iterations == 1000 and .benchmarks.aligned_compile_sweep.fields == 8 and .benchmarks.aligned_compile_sweep.compiled_lanes == 8 and .benchmarks.aligned_compile_sweep.ops_per_second > 0' <<<"$bench6m_aligned_compile_sweep_json" >/dev/null
bench6m_support_build_json="$("$bench6m" --mode support-build --support-build-iterations 10 --lane-sweep-width 8 --triads 16 --format json)"
jq -e '.benchmarks.support_build.iterations == 10 and .benchmarks.support_build.fields == 8 and .benchmarks.support_build.triads_in_memory == 16 and .benchmarks.support_build.ops_per_second > 0' <<<"$bench6m_support_build_json" >/dev/null
bench6m_support_build_compile_sweep_json="$("$bench6m" --mode support-build-compile-sweep --support-build-iterations 10 --lane-sweep-width 8 --triads 16 --format json)"
jq -e '.benchmarks.support_build_compile_sweep.iterations == 10 and .benchmarks.support_build_compile_sweep.fields == 8 and .benchmarks.support_build_compile_sweep.triads_in_memory == 16 and .benchmarks.support_build_compile_sweep.ops_per_second > 0' <<<"$bench6m_support_build_compile_sweep_json" >/dev/null
bench6m_support_score_build_json="$("$bench6m" --mode support-score-build --support-build-iterations 10 --lane-sweep-width 8 --triads 16 --format json)"
jq -e '.benchmarks.support_score_build.iterations == 10 and .benchmarks.support_score_build.fields == 8 and .benchmarks.support_score_build.triads_in_memory == 16 and .benchmarks.support_score_build.ops_per_second > 0' <<<"$bench6m_support_score_build_json" >/dev/null
bench6m_support_score_build_compile_sweep_json="$("$bench6m" --mode support-score-build-compile-sweep --support-build-iterations 10 --lane-sweep-width 8 --triads 16 --format json)"
jq -e '.benchmarks.support_score_build_compile_sweep.iterations == 10 and .benchmarks.support_score_build_compile_sweep.fields == 8 and .benchmarks.support_score_build_compile_sweep.triads_in_memory == 16 and .benchmarks.support_score_build_compile_sweep.ops_per_second > 0' <<<"$bench6m_support_score_build_compile_sweep_json" >/dev/null
bench6m_support_bucket_build_json="$("$bench6m" --mode support-bucket-build --support-build-iterations 10 --lane-sweep-width 8 --triads 16 --format json)"
jq -e '.benchmarks.support_bucket_build.iterations == 10 and .benchmarks.support_bucket_build.fields == 8 and .benchmarks.support_bucket_build.triads_in_memory == 16 and .benchmarks.support_bucket_build.ops_per_second > 0' <<<"$bench6m_support_bucket_build_json" >/dev/null
bench6m_support_bucket_build_compile_sweep_json="$("$bench6m" --mode support-bucket-build-compile-sweep --support-build-iterations 10 --lane-sweep-width 8 --triads 16 --format json)"
jq -e '.benchmarks.support_bucket_build_compile_sweep.iterations == 10 and .benchmarks.support_bucket_build_compile_sweep.fields == 8 and .benchmarks.support_bucket_build_compile_sweep.triads_in_memory == 16 and .benchmarks.support_bucket_build_compile_sweep.ops_per_second > 0' <<<"$bench6m_support_bucket_build_compile_sweep_json" >/dev/null
bench6m_hot_cycle_json="$("$bench6m" --mode hot-cycle --support-build-iterations 10 --lane-sweep-width 8 --triads 16 --format json)"
jq -e '.benchmarks.hot_cycle.iterations == 10 and .benchmarks.hot_cycle.fields == 8 and .benchmarks.hot_cycle.triads_in_memory == 16 and .benchmarks.hot_cycle.ops_per_second > 0' <<<"$bench6m_hot_cycle_json" >/dev/null
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
"$probe" --help | grep -q "Usage: nanda probe"
"$serve" --help | grep -q "Usage: nanda serve"
"$dogfood" --help | grep -q "Usage: nanda dogfood"
"$code_mapper" --help | grep -q "Usage: nanda map-code"
"$split_packet" --help | grep -q "Usage: nanda split"
"$reporter" --format md --title "Smoke Markdown Report" --domain code --overall "$root/examples/triads.watch-large.md" --route code:"$root/examples/triads.code-flow.md" >/dev/null || test "$?" -eq 3

echo "ok"
