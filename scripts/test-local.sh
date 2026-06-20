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
encode="$root/nanda-structural-gate/scripts/nanda-encode"
decode="$root/nanda-structural-gate/scripts/nanda-decode"
decode_eval="$root/nanda-structural-gate/scripts/nanda-decode-eval"
pattern_store="$root/nanda-structural-gate/scripts/nanda-pattern-store"
pattern_capacity="$root/nanda-structural-gate/scripts/nanda-pattern-capacity"
pattern_eval="$root/nanda-structural-gate/scripts/nanda-pattern-eval"
pattern_bank="$root/nanda-structural-gate/scripts/nanda-pattern-bank"
llmwave="$root/nanda-structural-gate/scripts/nanda-llmwave"
llmwave_eval="$root/nanda-structural-gate/scripts/nanda-llmwave-eval"
llmwave_memory="$root/nanda-structural-gate/scripts/nanda-llmwave-memory"
llmwave_big="$root/nanda-structural-gate/scripts/nanda-llmwave-big"
demo="$root/nanda-structural-gate/scripts/nanda-demo"
cache="$root/nanda-structural-gate/scripts/nanda-cache"
focus="$root/nanda-structural-gate/scripts/nanda-focus"
proof="$root/nanda-structural-gate/scripts/nanda-proof"
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
version_text="$("$root/target/debug/nanda" --version)"
grep -q '^nanda ' <<<"$version_text"
grep -q 'core_version: sparse-triad-v6.0-llmwave-proof' <<<"$version_text"
grep -q 'nanda_6m:' <<<"$version_text"
big_contract_json="$("$llmwave_big" contract --format json)"
jq -e '.roadmap_block == "v158-v160"' <<<"$big_contract_json" >/dev/null
jq -e '.contract.layers[] | select(.name == "L2 Word Field" and (.must_not_contain | index("schema_route_authority")))' <<<"$big_contract_json" >/dev/null
jq -e '.contract.layers[] | select(.name == "L3 Schema Field" and (.must_not_contain | index("surface_token_storage")))' <<<"$big_contract_json" >/dev/null
jq -e '.bigness_metrics.measured_baseline.status == "CONTRACT_BASELINE_ONLY_UNMEASURED_CORPUS"' <<<"$big_contract_json" >/dev/null
jq -e '.claim_boundary.current_state == "BIG_MODEL_NOT_PROVEN" and .claim_boundary.claims.llm_ready == false and .claim_boundary.claims.nonlinear_memory_proven == false' <<<"$big_contract_json" >/dev/null
big_atlas_json="$("$llmwave_big" atlas --format json)"
jq -e '.roadmap_block == "v161-v170" and .state == "ATLAS_CONTRACT_READY_NOT_HOT_RUNTIME"' <<<"$big_atlas_json" >/dev/null
jq -e '([.record_formats[].name] | index("SymbolAtom") and index("OperatorAtom") and index("SchemaRecord") and index("ResidualRecord"))' <<<"$big_atlas_json" >/dev/null
jq -e '.evidence_store.active_core_field == "evidence_ref" and (.active_packet_contract.must_not_contain | index("evidence_text"))' <<<"$big_atlas_json" >/dev/null
jq -e '.loader_preview.l2_projection == "surface_symbol_projection_only" and .loader_preview.l3_projection == "schema_operator_role_route_projection_only" and .loader_preview.fits_active_core_contract == true' <<<"$big_atlas_json" >/dev/null
jq -e '.doctor.verdict == "ATLAS_SAMPLE_OK"' <<<"$big_atlas_json" >/dev/null
big_active_core_json="$("$llmwave_big" active-core --format json)"
jq -e '.roadmap_block == "v171-v180" and .verdict == "ACTIVE_CORE_READY"' <<<"$big_active_core_json" >/dev/null
jq -e '.budget.total_bytes == 6291456 and .budget.fits_nanda_6m_budget == true' <<<"$big_active_core_json" >/dev/null
jq -e '.packet_format.schema_record_bytes == 32 and .packet_format.residual_record_bytes == 32 and .packet_format.lane_record_bytes == 64' <<<"$big_active_core_json" >/dev/null
jq -e '.cycle.top_schema_id == 101 and .cycle.safe_to_answer == true and .cycle.margin > 0' <<<"$big_active_core_json" >/dev/null
big_l2_json="$("$llmwave_big" l2 --format json)"
jq -e '.roadmap_block == "v181-v190" and .verdict == "L2_READY"' <<<"$big_l2_json" >/dev/null
jq -e '.candidate_cache.record_bytes == 32 and .candidate_cache.top_token_label == "invoice" and .candidate_cache.margin >= 12' <<<"$big_l2_json" >/dev/null
jq -e '.sync_policy.l2_update == "per_keystroke" and .sync_policy.l3_update == "word_boundary_punctuation_semantic_shift"' <<<"$big_l2_json" >/dev/null
jq -e '.candidate_cache.sample[] | select(.label == "inventory" and .anti_score > 0 and .final_score < 0)' <<<"$big_l2_json" >/dev/null
big_word_birth_json="$("$llmwave_big" word-birth --format json)"
jq -e '.roadmap_block == "v246-v252" and .verdict == "LEXICAL_BIRTH_MECHANISM_READY"' <<<"$big_word_birth_json" >/dev/null
jq -e '.sample.gate.verdict == "WORD_ACCEPTED" and .sample.binding_record.symbol_id == 70001' <<<"$big_word_birth_json" >/dev/null
jq -e '.rejection_control.gate.verdict == "WORD_REJECTED_OR_WAITING" and .rejection_control.binding_record == null' <<<"$big_word_birth_json" >/dev/null
jq -e '([.birth_stages[].stage] | index("segmentation") and index("cross_situational_convergence") and index("attractor_cleanup") and index("anti_confusion"))' <<<"$big_word_birth_json" >/dev/null
jq -e '([.record_formats[] | select(.bytes == 32) | .name] | index("LexicalBirthCandidate32") and index("LexicalBindingRecord32"))' <<<"$big_word_birth_json" >/dev/null
jq -e '.surface_production.primary_rule == "do_not_store_words_as_token_id_to_string; produce surfaces from composable form memory"' <<<"$big_word_birth_json" >/dev/null
jq -e '([.surface_production.production_layers[].layer] | index("grapheme_or_byte_atoms") and index("morpheme_atoms") and index("surface_program") and index("evidence_copy_span"))' <<<"$big_word_birth_json" >/dev/null
jq -e '.claim_boundary.corpus_proven == false and .claim_boundary.generator_ready == false and .claim_boundary.nonlinear_density_proven == false' <<<"$big_word_birth_json" >/dev/null
big_surface_json="$("$llmwave_big" surface-production --format json)"
jq -e '.roadmap_block == "v253-v260" and .verdict == "SURFACE_PRODUCTION_READY"' <<<"$big_surface_json" >/dev/null
jq -e '.production_law.primary_rule == "do_not_store_words_as_token_id_to_utf8; produce visible forms from composable surface memory"' <<<"$big_surface_json" >/dev/null
jq -e '([.record_formats[] | "\(.name):\(.bytes)"] | index("SurfaceAtom16:16") and index("SurfaceProgram32:32") and index("EvidenceCopySpan24:24") and index("SurfaceProductionCandidate32:32"))' <<<"$big_surface_json" >/dev/null
jq -e '.selected.production_path == "surface_program" and .selected.materialization_scope == "cold_report_preview_only_not_hot_storage"' <<<"$big_surface_json" >/dev/null
jq -e '([.atoms[].layer] | index("grapheme_or_byte_atoms") and index("morpheme_atoms")) and ([.programs[].path] | index("surface_program") and index("evidence_copy_span") and index("byte_fallback"))' <<<"$big_surface_json" >/dev/null
jq -e '.copy_spans[0].role == "exact rare form recovery" and .claim_boundary.real_corpus_trained == false and .claim_boundary.free_form_spelling_proven == false and .claim_boundary.nonlinear_surface_memory_proven == false' <<<"$big_surface_json" >/dev/null
big_write_json="$("$llmwave_big" write --format json)"
jq -e '.roadmap_block == "v191-v205" and .verdict == "RESIDUAL_SAVING"' <<<"$big_write_json" >/dev/null
jq -e '.residual_format_v1.bytes == 20 and .write_decision.bytes_written == 28' <<<"$big_write_json" >/dev/null
jq -e '.write_curve.state == "SYNTHETIC_CONTRACT_CURVE_NOT_NONLINEAR_PROOF" and .write_curve.residual_saving_ratio > 0.5' <<<"$big_write_json" >/dev/null
jq -e '.compression_safety.safe == true and .anti_residual.anti_lane_id == 90001' <<<"$big_write_json" >/dev/null
big_consolidate_json="$("$llmwave_big" consolidate --format json)"
jq -e '.roadmap_block == "v206-v218" and .verdict == "CONSOLIDATION_SAFE"' <<<"$big_consolidate_json" >/dev/null
jq -e '.conflict_preservation.state == "CONFLICTS_PRESERVED" and .eval.safe == true' <<<"$big_consolidate_json" >/dev/null
jq -e '.eval.after.memory_bytes < .eval.before.memory_bytes and .eval.after.role_safety >= .eval.before.role_safety' <<<"$big_consolidate_json" >/dev/null
jq -e '.anti_memory.anti_lanes_created == 1 and .cognitive_compression_score > 1' <<<"$big_consolidate_json" >/dev/null
big_eval_json="$("$llmwave_big" eval --format json)"
jq -e '.roadmap_block == "v219-v230" and .verdict == "COGNITIVE_LIFT"' <<<"$big_eval_json" >/dev/null
jq -e '(.cases | length) == 9 and ([.cases[].task_type] | index("role_swap") and index("contradiction") and index("multi_hop") and index("business"))' <<<"$big_eval_json" >/dev/null
jq -e '.cognitive_score.total >= 0.8 and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_eval_json" >/dev/null
jq -e '.claim_boundary.candidate_ready == false' <<<"$big_eval_json" >/dev/null
big_query_json="$("$llmwave_big" query --text "supplier invoice payment customs" --format json)"
jq -e '.roadmap_block == "v231-v245" and .verdict == "LLMWAVE_BIG_V1_CANDIDATE"' <<<"$big_query_json" >/dev/null
jq -e '.safety.safe_to_answer == true and .skill_integration.state == "CLI_SURFACE_READY"' <<<"$big_query_json" >/dev/null
jq -e '.v1_criteria.large_long_term_memory == true and .v1_criteria.schema_residual_write == true and .claim_boundary.llm_ready == false' <<<"$big_query_json" >/dev/null
big_query_contested_json="$("$llmwave_big" query --text "role swap conflict" --format json)"
jq -e '.safety.field_state == "FIELD_CONTESTED" and .safety.safe_to_answer == false' <<<"$big_query_contested_json" >/dev/null
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
jq empty "$root/examples/triad-packet.demo-review-empty.json"
jq empty "$root/examples/triad-packet.canonical-alias-pass.json"
jq empty "$root/examples/triad-packet.canonical-alias-veto.json"
jq empty "$root/examples/triad-packet.canonical-alias-conflict.json"
jq empty "$root/examples/eval-corpus.json"
jq empty "$root/examples/probe-corpus.json"
jq empty "$root/examples/waw-corpus.json"
jq empty "$root/examples/decode-corpus.json"
jq empty "$root/examples/pattern-learning-corpus.json"
jq empty "$root/examples/llmwave-corpus.json"
jq empty "$root/examples/token-lens-corpus.json"
jq empty "$root/examples/llmwave-memory-corpus.json"
jq -e 'length > 0' <<<"$(jq -Rs . "$root/examples/llmwave-tiny-corpus.txt")" >/dev/null
jq empty "$root/examples/triad-packet.token-lens-business.json"
jq empty "$root/examples/demo-corpus.json"

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
grep -q '"core_version": "sparse-triad-v6.0-llmwave-proof"' <<<"$map_json"
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

tmp_proof_report="$(mktemp)"
tmp_proof_focus="$(mktemp)"
set +e
proof_json="$("$proof" "$root/examples/triad-packet.route-balanced-focus.json" --input-format json --max-triads 12 --route-cap 4 --route-triad-cap 4 --focus-out "$tmp_proof_focus" --out "$tmp_proof_report")"
proof_status=$?
set -e
if [[ "$proof_status" -ne 0 && "$proof_status" -ne 3 ]]; then
  echo "expected proof to PASS or WATCH" >&2
  echo "$proof_json" >&2
  exit 1
fi
jq -e '.mode == "proof-from-focus"' <<<"$proof_json" >/dev/null
jq -e '.proof_mode == "full-compare"' <<<"$proof_json" >/dev/null
jq -e '.proof_version == "v27-proof-reason-suite"' <<<"$proof_json" >/dev/null
jq -e '.reason_codes | length >= 1' <<<"$proof_json" >/dev/null
jq -e '.proof_confidence.score >= 0' <<<"$proof_json" >/dev/null
jq -e '.proof_compare.state | length > 0' <<<"$proof_json" >/dev/null
jq -e '.focused_memory_size <= 12' <<<"$proof_json" >/dev/null
jq -e '.runtime_contract.focus.state == "PACKED_RUNTIME_READY"' <<<"$proof_json" >/dev/null
jq empty "$tmp_proof_report"
jq empty "$tmp_proof_focus"
rm -f "$tmp_proof_report" "$tmp_proof_focus"
tmp_cache="$(mktemp -d)"
cache_build_json="$("$cache" build "$root/examples/triad-packet.route-balanced-focus.json" --input-format json --query "lower operator debt route" --out-dir "$tmp_cache")"
jq -e '.mode == "focus-cache-build" and .version == "v64-focus-cache" and .focused_memory_size > 0' <<<"$cache_build_json" >/dev/null
cache_list_json="$("$cache" list "$tmp_cache")"
jq -e '.mode == "focus-cache-list" and .version == "v65-cache-only-proof" and .count == 1' <<<"$cache_list_json" >/dev/null
cache_proof_json="$("$proof" "$root/examples/triad-packet.route-balanced-focus.json" --input-format json --query "lower operator debt route" --fast --cache-dir "$tmp_cache")"
jq -e '.proof_mode == "fast-focused" and .focus_cache.state == "CACHE_HIT" and (.reason_codes | index("RAW_SEARCH_SKIPPED"))' <<<"$cache_proof_json" >/dev/null
cache_manifest="$(find "$tmp_cache" -type f -name '*.manifest.json' | head -n 1)"
cache_only_json="$("$proof" --cache-only "$cache_manifest")"
jq -e '.proof_mode == "cache-only-focused" and .focus_cache.state == "CACHE_ONLY_HIT" and .corpus.state == "CORPUS_NOT_LOADED" and (.reason_codes | index("CORPUS_NOT_LOADED"))' <<<"$cache_only_json" >/dev/null
serve_cache_json="$(printf '{"command":"proof_cache_only","manifest":"%s"}\n{"command":"proof_cache_only","manifest":"%s"}\n' "$cache_manifest" "$cache_manifest" | "$serve")"
jq -e 'length == 2 and .[0].ok == true and .[0].elapsed_ms >= 0 and .[0].result.proof_mode == "cache-only-focused" and .[0].result.focus_cache.state == "CACHE_ONLY_HIT" and .[0].result.serve_cache.state == "SERVE_MEMORY_WARMED" and .[1].ok == true and .[1].result.serve_cache.state == "SERVE_PROOF_HIT"' <<<"$(jq -s . <<<"$serve_cache_json")" >/dev/null
serve_compact_json="$(printf '{"command":"proof_cache_only","manifest":"%s","response":"compact"}\n{"command":"proof_cache_only","manifest":"%s","response":"compact"}\n' "$cache_manifest" "$cache_manifest" | "$serve")"
jq -e 'length == 2 and .[0].ok == true and .[0].result.mode == "proof-cache-only-compact" and ((.[0].result.proof_state | length) > 0) and .[0].result.focused_search == null and .[1].result.serve_cache.state == "SERVE_PROOF_HIT"' <<<"$(jq -s . <<<"$serve_compact_json")" >/dev/null
rm -rf "$tmp_cache"
proof_suite_json="$("$proof" --suite "$root/examples/proof-corpus.json" --input-format json)"
jq -e '.mode == "proof-suite"' <<<"$proof_suite_json" >/dev/null
jq -e '.proof_version == "v27-proof-reason-suite"' <<<"$proof_suite_json" >/dev/null
jq -e '.passed == .total' <<<"$proof_suite_json" >/dev/null
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
bench6m_active_core_json="$("$bench6m" --mode active-core --support-build-iterations 1000 --format json)"
jq -e '.benchmarks.active_core.mode == "llmwave-big-active-core" and .benchmarks.active_core.iterations == 1000' <<<"$bench6m_active_core_json" >/dev/null
jq -e '.benchmarks.active_core.ns_per_query > 0 and .benchmarks.active_core.checksum != 0' <<<"$bench6m_active_core_json" >/dev/null
bench6m_write_json="$("$bench6m" --mode write-density --support-build-iterations 1000 --format json)"
jq -e '.benchmarks.write_density.mode == "llmwave-big-write-density" and .benchmarks.write_density.iterations == 1000' <<<"$bench6m_write_json" >/dev/null
jq -e '.benchmarks.write_density.ns_per_write > 0 and .benchmarks.write_density.checksum != 0' <<<"$bench6m_write_json" >/dev/null
bench6m_consolidate_json="$("$bench6m" --mode consolidate --support-build-iterations 1000 --format json)"
jq -e '.benchmarks.consolidate.mode == "llmwave-big-consolidate" and .benchmarks.consolidate.iterations == 1000' <<<"$bench6m_consolidate_json" >/dev/null
jq -e '.benchmarks.consolidate.ns_per_pass > 0 and .benchmarks.consolidate.checksum != 0' <<<"$bench6m_consolidate_json" >/dev/null
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
jq -e '.resonant_field.version == "v28-resonant-field-core"' <<<"$trap_search_json" >/dev/null
jq -e '.resonant_field.state == "WAW_RESONANCE" and .resonant_field.waw_status == "WAW_RESONANCE"' <<<"$trap_search_json" >/dev/null
jq -e '.resonant_field.phase_lock.state == "PHASE_LOCKED"' <<<"$trap_search_json" >/dev/null
jq -e '.resonant_field.standing_wave.state == "STANDING_STABLE"' <<<"$trap_search_json" >/dev/null
jq -e '.resonant_field.energy.state == "ENERGY_CONTAINED"' <<<"$trap_search_json" >/dev/null
jq -e '.field_interpretation.lexical_trap_detected == true' <<<"$trap_search_json" >/dev/null
jq -e '.field_interpretation.centroid_drift.route.changed == true' <<<"$trap_search_json" >/dev/null
decode_json="$("$decode" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --top-k 5)"
jq -e '.mode == "wave-pattern-decoder"' <<<"$decode_json" >/dev/null
jq -e '.decoder_version == "v30-pattern-store-wave-decoder"' <<<"$decode_json" >/dev/null
jq -e '.decoder_state == "PATTERN_READY" and .safe_to_generate == true' <<<"$decode_json" >/dev/null
jq -e '.source_search.top_peak == "certification"' <<<"$decode_json" >/dev/null
jq -e '.patterns | length >= 3' <<<"$decode_json" >/dev/null
jq -e '.patterns[0].decode_as == "next_structural_pattern"' <<<"$decode_json" >/dev/null
jq -e '.patterns[0].subject_role != "" and .patterns[0].object_role != ""' <<<"$decode_json" >/dev/null
decode_steps_json="$("$decode" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --top-k 3 --steps 3)"
jq -e '.decoder_version == "v31-recurrent-wave-decoder"' <<<"$decode_steps_json" >/dev/null
jq -e '.recurrent.enabled == true and .recurrent.requested_steps == 3' <<<"$decode_steps_json" >/dev/null
jq -e '.recurrent.completed_steps >= 2' <<<"$decode_steps_json" >/dev/null
jq -e '.recurrent.steps[0].selected_pattern.decode_as == "next_structural_pattern"' <<<"$decode_steps_json" >/dev/null
jq -e '.recurrent.steps[-1].decoder_state == "PATTERN_READY" or .recurrent.steps[-1].decoder_state == "PATTERN_SATURATED"' <<<"$decode_steps_json" >/dev/null
jq -e '.energy_trace.version == "v49-attractor-energy-trace" and .energy_trace.state != "NO_ENERGY_TRACE"' <<<"$decode_steps_json" >/dev/null
decode_eval_json="$("$decode_eval" --suite "$root/examples/decode-corpus.json")"
jq -e '.mode == "decode-eval-suite" and .passed == 3 and .total == 3 and .accuracy == 1' <<<"$decode_eval_json" >/dev/null
jq -e '.cases[] | select(.id == "route-trap-recurrent-saturates" and .actual_final_decoder_state == "PATTERN_SATURATED")' <<<"$decode_eval_json" >/dev/null
jq -e '.cases[] | select(.id == "route-trap-beam-trajectory" and .actual_beam_route == "certification" and .actual_beam_length >= 2 and .forbidden_seen == false)' <<<"$decode_eval_json" >/dev/null
beam_decode_json="$("$decode" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --top-k 3 --steps 3 --beam-width 3 --adaptive-scoring)"
jq -e '.decoder_version == "v42-beam-wave-decoder" and .beam_decode.enabled == true and .beam_decode.trajectories[0].route_center == "certification"' <<<"$beam_decode_json" >/dev/null
jq -e '.recurrent.steps[0].adaptive_pattern_scoring.version == "v45-adaptive-pattern-scoring" and .recurrent.steps[0].adaptive_pattern_scoring.enabled == true' <<<"$beam_decode_json" >/dev/null
tmp_decode_feedback="$(mktemp)"
tmp_decode_training_feedback="$(mktemp)"
tmp_decode_index="$(mktemp)"
printf '%s\n' "$decode_json" >"$tmp_decode_feedback"
"$feedback" "$tmp_decode_feedback" --decision accept --note "accepted continuation" --out "$tmp_decode_training_feedback" >/dev/null
jq -e '.source_mode == "wave-pattern-decoder"' "$tmp_decode_training_feedback" >/dev/null
jq -e '.continuation_memory[0].decision == "accept"' "$tmp_decode_training_feedback" >/dev/null
jq -e '.continuation_memory[0].subject == "declaration" and .continuation_memory[0].relation == "requires" and .continuation_memory[0].object == "protocols"' "$tmp_decode_training_feedback" >/dev/null
"$indexer" "$root/examples/triad-packet.interference-search-route-trap.json" "$tmp_decode_training_feedback" --input-format json --out "$tmp_decode_index" >/dev/null
jq -e '.continuation_memory[0].accepted_count == 1' "$tmp_decode_index" >/dev/null
trained_decode_json="$("$decode" "$tmp_decode_index" --input-format json --query-file "$root/examples/triad-packet.interference-search-route-trap.json" --query-format json --top-k 3)"
jq -e '.continuation_training.applied == true' <<<"$trained_decode_json" >/dev/null
jq -e '.patterns[0].continuation_memory_delta > 0' <<<"$trained_decode_json" >/dev/null
jq -e '.compact_pattern_store.version == "v35-compact-pattern-store" and .continuation_training.version == "v36-pattern-replay"' <<<"$trained_decode_json" >/dev/null
jq -e '.recurrent.steps[0].early_pattern_replay.version == "v44-pre-ranking-pattern-replay"' <<<"$trained_decode_json" >/dev/null
pattern_store_json="$("$pattern_store" "$tmp_decode_index" --input-format json)"
jq -e '.mode == "compact-pattern-store" and .packed_pattern_bytes == 32 and .records == 1 and .fits_pattern_arena == true' <<<"$pattern_store_json" >/dev/null
pattern_bank_json="$("$pattern_bank" "$tmp_decode_index" --input-format json --mode inspect)"
jq -e '.mode == "pattern-bank" and .version == "v48-cleanup-pattern-bank" and .records == 1 and .fits_pattern_arena == true' <<<"$pattern_bank_json" >/dev/null
jq -e '.cleanup_memory.version == "v48-cleanup-memory" and .cleanup_memory.state == "CLEANUP_DICTIONARY_READY"' <<<"$pattern_bank_json" >/dev/null
pattern_bank_apply_json="$("$pattern_bank" "$tmp_decode_index" --input-format json --mode apply)"
jq -e '.apply_state == "PATTERN_BANK_READY_FOR_DECODE"' <<<"$pattern_bank_apply_json" >/dev/null
pattern_capacity_json="$("$pattern_capacity")"
jq -e '.mode == "llmwave-pattern-capacity" and .pattern_store_capacity == 16384' <<<"$pattern_capacity_json" >/dev/null
jq -e '.rows[] | select(.patterns == 65536 and .fits_pattern_arena == false)' <<<"$pattern_capacity_json" >/dev/null
pattern_eval_json="$("$pattern_eval" --suite "$root/examples/pattern-learning-corpus.json")"
jq -e '.mode == "pattern-learning-eval-suite" and .version == "v41-learning-effect-eval" and .passed == 2 and .total == 2 and .accuracy == 1' <<<"$pattern_eval_json" >/dev/null
jq -e '.learning_effect.changed_top == 1 and .learning_effect.reinforced_same_top == 1' <<<"$pattern_eval_json" >/dev/null
jq -e '.cases[] | select(.id == "reject-top-continuation-changes-ranking" and .learning_changed_top == true and .actual_action == "suppress")' <<<"$pattern_eval_json" >/dev/null
jq -e '.cases[] | select(.id == "accept-top-continuation-reinforces-score" and .training_applied == true and .actual_action == "reinforce")' <<<"$pattern_eval_json" >/dev/null
llmwave_json="$("$llmwave" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "declaration requires protocols" --train)"
jq -e '.mode == "llmwave-mini-loop" and .version == "v60-public-demo-packet"' <<<"$llmwave_json" >/dev/null
jq -e '.hrr_binding.version == "v47-hrr-binding-sandbox" and .hrr_binding.state == "HRR_BINDING_VISIBLE"' <<<"$llmwave_json" >/dev/null
jq -e '.cleanup_memory.version == "v48-cleanup-memory" and .cleanup_memory.state != "CLEANUP_WATCH"' <<<"$llmwave_json" >/dev/null
jq -e '.attractor_trace.version == "v49-attractor-energy-trace" and .attractor_trace.state != "NO_ATTRACTOR_TRACE"' <<<"$llmwave_json" >/dev/null
jq -e '.superposition_capacity.version == "v50-superposition-capacity-curve" and .superposition_capacity.state != "FOCUS_REQUIRED"' <<<"$llmwave_json" >/dev/null
jq -e '.anti_wave_audit.version == "v51-shortcut-specific-anti-wave-audit"' <<<"$llmwave_json" >/dev/null
jq -e '.packed_hrr_runtime.version == "v54-packed-hrr-lanes" and .packed_hrr_runtime.state == "PACKED_HRR_READY"' <<<"$llmwave_json" >/dev/null
jq -e '.cleanup_dictionary.version == "v55-cleanup-dictionary-thresholds" and .cleanup_dictionary.state == "CLEANUP_DICTIONARY_READY"' <<<"$llmwave_json" >/dev/null
jq -e '.anti_wave_locality.version == "v56-anti-wave-locality-fixture"' <<<"$llmwave_json" >/dev/null
jq -e '.capacity_curve.version == "v57-superposition-capacity-baseline" and .capacity_curve.state != "FOCUS_REQUIRED"' <<<"$llmwave_json" >/dev/null
jq -e '.packed_hot_cycle.version == "v58-packed-hot-cycle-bridge" and .packed_hot_cycle.state == "LLMWAVE_HOT_READY"' <<<"$llmwave_json" >/dev/null
jq -e '.proof_summary.version == "v59-llmwave-proof-command-contract" and .proof_summary.state == "LLMWAVE_PROOF_READY"' <<<"$llmwave_json" >/dev/null
jq -e '.public_demo.version == "v60-public-demo-packet" and .public_demo.state == "PUBLIC_DEMO_READY"' <<<"$llmwave_json" >/dev/null
jq -e '.llmwave_contract.version == "v67-field-lens-contract" and .llmwave_contract.state == "LLMWAVE_LENS_READY" and .llmwave_contract.selected == "pattern" and .llmwave_contract.lenses.pattern.state == "PATTERN_LENS_READY" and .llmwave_contract.lenses.cleanup.state == "CLEANUP_LENS_EXACT"' <<<"$llmwave_json" >/dev/null
jq -e '.decode.top_pattern == "declaration -> requires -> protocols" and .feedback_preview.enabled == true' <<<"$llmwave_json" >/dev/null
llmwave_polarity_json="$("$llmwave" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "declaration requires protocols" --lens polarity)"
jq -e '.llmwave_contract.selected == "polarity" and .llmwave_contract.selected_lens.state == "POLARITY_LENS_DIRECTIONAL" and .llmwave_contract.state == "LLMWAVE_LENS_READY"' <<<"$llmwave_polarity_json" >/dev/null
llmwave_token_json="$("$llmwave" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "customs declaration requires" --lens token)"
jq -e '.llmwave_contract.selected == "token" and .llmwave_contract.selected_lens.version == "v75-token-lens-next-token-resonance" and .llmwave_contract.selected_lens.top_token == "payment" and .llmwave_contract.selected_lens.token_cleanup.state == "TOKEN_CLEANUP_EXACT"' <<<"$llmwave_token_json" >/dev/null
llmwave_convex_json="$("$llmwave" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "declaration requires protocols" --lens convex)"
jq -e '.llmwave_contract.selected == "convex" and .llmwave_contract.selected_lens.version == "v78-convex-gathering-lens" and .llmwave_contract.selected_lens.state == "CONVEX_LENS_READY" and .llmwave_contract.selected_lens.top_basin == "certification" and .llmwave_contract.field_snapshot.version == "v77-field-snapshot"' <<<"$llmwave_convex_json" >/dev/null
llmwave_concave_json="$("$llmwave" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "declaration requires protocols" --lens concave)"
jq -e '.llmwave_contract.selected == "concave" and .llmwave_contract.selected_lens.version == "v79-concave-separation-lens" and .llmwave_contract.selected_lens.state == "CONCAVE_LENS_SPLIT" and .llmwave_contract.selected_lens.competing_branches >= 2' <<<"$llmwave_concave_json" >/dev/null
llmwave_prism_json="$("$llmwave" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "declaration requires protocols" --lens prism)"
jq -e '.llmwave_contract.selected == "prism" and .llmwave_contract.selected_lens.version == "v80-prism-explanation-lens" and .llmwave_contract.selected_lens.state == "PRISM_LENS_READY" and .llmwave_contract.selected_lens.contributions.routes[0].key == "certification" and .llmwave_contract.lens_taxonomy.version == "v76-lens-taxonomy"' <<<"$llmwave_prism_json" >/dev/null
llmwave_role_json="$("$llmwave" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "declaration requires protocols" --lens role)"
jq -e '.llmwave_contract.selected == "role" and .llmwave_contract.selected_lens.version == "v81-role-binding-lens" and .llmwave_contract.selected_lens.state == "ROLE_LENS_READY" and .llmwave_contract.selected_lens.top_role_path == "document->evidence->evidence"' <<<"$llmwave_role_json" >/dev/null
llmwave_temporal_json="$("$llmwave" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "declaration requires protocols" --lens temporal)"
jq -e '.llmwave_contract.selected == "temporal" and .llmwave_contract.selected_lens.version == "v82-temporal-order-lens" and .llmwave_contract.selected_lens.state == "TEMPORAL_LENS_ORDERED" and .llmwave_contract.selected_lens.route_jumps == 0' <<<"$llmwave_temporal_json" >/dev/null
llmwave_evidence_json="$("$llmwave" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "declaration requires protocols" --lens evidence)"
jq -e '.llmwave_contract.selected == "evidence" and .llmwave_contract.selected_lens.version == "v83-evidence-binding-lens" and .llmwave_contract.selected_lens.state == "EVIDENCE_LENS_READY" and .llmwave_contract.selected_lens.top_evidence_bound == true' <<<"$llmwave_evidence_json" >/dev/null
llmwave_energy_json="$("$llmwave" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "declaration requires protocols" --lens energy)"
jq -e '.llmwave_contract.selected == "energy" and .llmwave_contract.selected_lens.version == "v84-energy-stability-lens" and .llmwave_contract.selected_lens.state == "ENERGY_LENS_CONTESTED" and .llmwave_contract.selected_lens.final_energy > 0' <<<"$llmwave_energy_json" >/dev/null
tmp_anti_packet="$(mktemp)"
jq '.continuation_memory=[{"id":"cont-reject-protocols","decision":"reject","pattern_id":"pat-declaration-requires-protocols","subject":"declaration","relation":"requires","object":"protocols","route":"certification","group":"certification-route","peak":"certification","penalty":0.18,"reason":"test reject","source_feedback":"test","observations":1,"rejected_count":1}]' "$root/examples/triad-packet.interference-search-route-trap.json" >"$tmp_anti_packet"
llmwave_anti_json="$("$llmwave" "$tmp_anti_packet" --input-format json --text "declaration requires protocols" --lens anti)"
jq -e '.llmwave_contract.selected == "anti" and .llmwave_contract.selected_lens.version == "v85-anti-lens-destructive-report" and .llmwave_contract.selected_lens.state == "ANTI_LENS_SUPPRESSED_SHORTCUT" and (.llmwave_contract.selected_lens.suppressions | length) == 1' <<<"$llmwave_anti_json" >/dev/null
llmwave_eval_json="$("$llmwave_eval" --suite "$root/examples/llmwave-corpus.json")"
jq -e '.mode == "llmwave-eval-suite" and .version == "v53-llmwave-proof-suite" and .passed == 2 and .total == 2 and .accuracy == 1' <<<"$llmwave_eval_json" >/dev/null
jq -e '.cases[].states.llmwave_contract == "LLMWAVE_LENS_READY"' <<<"$llmwave_eval_json" >/dev/null
jq -e '.cases[] | select(.id == "route-trap-reject-applies-anti-wave" and .states.anti_wave == "ANTI_WAVE_APPLIED")' <<<"$llmwave_eval_json" >/dev/null
token_lens_eval_json="$("$llmwave_eval" --suite "$root/examples/token-lens-corpus.json")"
jq -e '.mode == "llmwave-eval-suite" and .passed == 5 and .total == 5 and .accuracy == 1' <<<"$token_lens_eval_json" >/dev/null
jq -e '.cases[] | select(.id == "reject-protocols-shifts-token" and .actual_next_token == "payment" and .states.anti_wave == "ANTI_WAVE_APPLIED")' <<<"$token_lens_eval_json" >/dev/null
tmp_memory="$(mktemp -d)"
"$llmwave_memory" write "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "customs declaration requires payment" --out "$tmp_memory/memory.json"
jq -e '.version == "v86-wave-memory-schema" and .write_path.version == "v87-memory-write-path" and .wave_memory.phrase_memory.version == "v92-phrase-memory" and .wave_memory.packed_runtime.version == "v93-packed-6m-memory"' "$tmp_memory/memory.json" >/dev/null
memory_vocab_json="$("$llmwave_memory" vocabulary "$tmp_memory/memory.json")"
jq -e '.mode == "llmwave-memory-vocabulary" and .version == "v96-vocabulary-token-space" and .tokens > 0' <<<"$memory_vocab_json" >/dev/null
memory_inspect_json="$("$llmwave_memory" inspect "$tmp_memory/memory.json")"
jq -e '.mode == "llmwave-memory-inspect" and .version == "v105-real-memory-file-format" and .tokenizer_contract.version == "v106-tokenizer-contract" and .model_config.version == "v107-model-config"' <<<"$memory_inspect_json" >/dev/null
memory_pack_json="$("$llmwave_memory" pack "$tmp_memory/memory.json" --out "$tmp_memory/memory.llmw.bin")"
jq -e '.mode == "llmwave-memory-pack" and .version == "v108-binary-packed-memory-prototype" and .bytes > 16' <<<"$memory_pack_json" >/dev/null
memory_unpack_json="$("$llmwave_memory" unpack "$tmp_memory/memory.llmw.bin")"
jq -e '.mode == "llmwave-memory-unpack" and .version == "v108-binary-packed-memory-prototype" and .state == "PACKED_MEMORY_OK"' <<<"$memory_unpack_json" >/dev/null
memory_retrieve_json="$("$llmwave_memory" retrieve "$tmp_memory/memory.json" --prefix "customs declaration requires")"
jq -e '.mode == "llmwave-memory-retrieve" and .version == "v88-memory-retrieve-path" and .state == "MEMORY_RETRIEVE_READY" and .top_token == "payment" and .packed_runtime.fits_6m == true' <<<"$memory_retrieve_json" >/dev/null
memory_feedback_json="$("$llmwave_memory" feedback "$tmp_memory/memory.json" --decision reject --token protocols)"
jq -e '.mode == "llmwave-memory-feedback" and .version == "v89-feedback-learning" and .state == "MEMORY_FEEDBACK_APPLIED" and .touched >= 1' <<<"$memory_feedback_json" >/dev/null
memory_correct_json="$("$llmwave_memory" correct "$tmp_memory/memory.json" --reject-token protocols --accept-token payment)"
jq -e '.mode == "llmwave-memory-correct" and .version == "v103-self-correction" and .state == "MEMORY_CORRECTION_APPLIED" and (.actions | length) == 2' <<<"$memory_correct_json" >/dev/null
jq '.memory' <<<"$memory_feedback_json" >"$tmp_memory/memory-feedback.json"
memory_consolidate_json="$("$llmwave_memory" consolidate "$tmp_memory/memory-feedback.json")"
jq -e '.mode == "llmwave-memory-consolidate" and .version == "v90-consolidation" and .memory.wave_memory.consolidation.state == "CONSOLIDATED"' <<<"$memory_consolidate_json" >/dev/null
jq '.memory' <<<"$memory_consolidate_json" >"$tmp_memory/memory-consolidated.json"
memory_decay_json="$("$llmwave_memory" decay "$tmp_memory/memory-consolidated.json" --factor 0.99)"
jq -e '.mode == "llmwave-memory-decay" and .version == "v91-decay-forgetting" and .memory.wave_memory.decay.state == "DECAY_APPLIED"' <<<"$memory_decay_json" >/dev/null
memory_generate_json="$("$llmwave_memory" generate "$tmp_memory/memory.json" --prefix "customs declaration requires" --steps 2 --beam-width 2 --temperature 0)"
jq -e '.mode == "llmwave-memory-generate" and .version == "v94-recurrent-generation" and .coherence.version == "v112-multi-step-coherence" and (.generated_text | contains("payment")) and .steps[0].sampler.version == "v97-sampler" and .steps[0].beams[0].version == "v98-beam-generator" and .steps[0].beams[0].semantic_guard.version == "v111-semantic-guard" and .semantic_decoder.version == "v99-semantic-decoder"' <<<"$memory_generate_json" >/dev/null
memory_chat_json="$("$llmwave_memory" chat "$tmp_memory/memory.json" --prompt "what does customs declaration require?" --steps 2)"
jq -e '.mode == "llmwave-memory-chat" and .version == "v100-chat-loop" and .prompt_adapter.version == "v110-prompt-adapter" and .prompt_adapter.selected_prefix == "customs declaration requires" and (.answer | contains("payment"))' <<<"$memory_chat_json" >/dev/null
memory_answer_json="$("$llmwave_memory" answer "$tmp_memory/memory.json" --prompt "what does customs declaration require?" --facts 3)"
jq -e '.mode == "llmwave-memory-answer" and .version == "v115-answer-contract" and .answer_versions.grounding == "v116-grounded-answer" and .answer_versions.multi_fact == "v117-multi-fact-answer" and .answer_versions.review_state == "v118-answer-review-state" and .core_versions.relation_phase == "v120-relation-phase-channels" and .core_versions.polarity == "v121-subject-object-polarity" and .core_versions.bidirectional_recall == "v122-bidirectional-recall" and .field_core.version == "v124-phase-collision-detector" and .state == "ANSWER_READY" and .safe_to_answer == true and (.answer | contains("payment")) and (.grounding.facts | length) >= 1' <<<"$memory_answer_json" >/dev/null
printf 'customs declaration requires payment confirmation. payment confirmation supports customs declaration.\n' >"$tmp_memory/train.txt"
"$llmwave_memory" train "$tmp_memory/train.txt" --out "$tmp_memory/train-memory.json"
jq -e '.version == "v101-training-from-text" and .write_path.state == "TEXT_MEMORY_WRITTEN" and .vocabulary.version == "v96-vocabulary-token-space"' "$tmp_memory/train-memory.json" >/dev/null
memory_grow_json="$("$llmwave_memory" grow "$tmp_memory/memory.json" "$root/examples/triad-packet.token-lens-business.json" --input-format json)"
jq -e '.mode == "llmwave-memory-grow" and .version == "v102-memory-growth" and .after_records > .before_records' <<<"$memory_grow_json" >/dev/null
memory_eval_json="$("$llmwave_memory" eval --suite "$root/examples/llmwave-memory-corpus.json")"
jq -e '.mode == "llmwave-memory-eval" and .version == "v126-core-field-eval" and .legacy_version == "v119-qa-answer-eval" and .passed == 10 and .total == 10 and .accuracy == 1 and (.cases[] | select(.id == "qa-reversed-invoice-issue-veto" and .answer_state == "ANSWER_EMPTY" and .ok == true))' <<<"$memory_eval_json" >/dev/null
serve_token_json="$(printf '{"command":"llmwave_token","input":"%s/examples/triad-packet.interference-search-route-trap.json","text":"customs declaration requires"}\n{"command":"llmwave_token","input":"%s/examples/triad-packet.interference-search-route-trap.json","text":"customs declaration requires"}\n' "$root" "$root" | "$serve")"
jq -e 'length == 2 and .[0].ok == true and .[0].result.mode == "llmwave-token-compact" and .[0].result.top_token == "payment" and .[0].result.serve_cache.state == "SERVE_TOKEN_WARMED" and .[1].result.serve_cache.state == "SERVE_TOKEN_HIT"' <<<"$(jq -s . <<<"$serve_token_json")" >/dev/null
serve_chat_json="$(printf '{"command":"llmwave_chat","memory":"%s","prompt":"what does customs declaration require?","steps":2}\n{"command":"llmwave_chat","memory":"%s","prompt":"what does customs declaration require?","steps":2}\n' "$tmp_memory/memory.json" "$tmp_memory/memory.json" | "$serve")"
jq -e 'length == 2 and .[0].ok == true and .[0].result.mode == "llmwave-memory-chat" and .[0].result.prompt_adapter.version == "v110-prompt-adapter" and .[0].result.serve_cache.state == "SERVE_CHAT_WARMED" and .[1].result.serve_cache.state == "SERVE_CHAT_HIT"' <<<"$(jq -s . <<<"$serve_chat_json")" >/dev/null
serve_answer_json="$(printf '{"command":"llmwave_answer","memory":"%s","prompt":"what does customs declaration require?","facts":3}\n{"command":"llmwave_answer","memory":"%s","prompt":"what does customs declaration require?","facts":3}\n' "$tmp_memory/memory.json" "$tmp_memory/memory.json" | "$serve")"
jq -e 'length == 2 and .[0].ok == true and .[0].result.mode == "llmwave-memory-answer" and .[0].result.state == "ANSWER_READY" and .[0].result.serve_cache.state == "SERVE_ANSWER_WARMED" and .[1].result.serve_cache.state == "SERVE_ANSWER_HIT"' <<<"$(jq -s . <<<"$serve_answer_json")" >/dev/null
memory_demo_json="$("$llmwave_memory" demo --corpus "$root/examples/llmwave-tiny-corpus.txt" --prompt "what does customs declaration require?")"
jq -e '.mode == "llmwave-memory-demo" and .version == "v114-public-demo-script" and .state == "LLMWAVE_MEMORY_DEMO_READY" and .after.prompt_adapter.version == "v110-prompt-adapter" and .after.generation.coherence.version == "v112-multi-step-coherence" and .packed.state == "PACKED_MEMORY_OK"' <<<"$memory_demo_json" >/dev/null
memory_density_json="$("$llmwave_memory" density --counts 16,64,256 --facts 3)"
jq -e '.mode == "llmwave-memory-density" and .version == "v127-density-reality-check" and .claims_boundary.nonlinear_density_proven == false and .claims_boundary.lexical_baseline_compared == true and .density_reader.version == "v138-density-report-reader" and .stress_pack.version == "v139-baseline-stress-pack" and .margin_erosion_curve.version == "v140-margin-erosion-curve" and .fixed_basis_test.version == "v141-fixed-basis-test" and .useful_capacity_threshold.version == "v142-useful-capacity-threshold" and .anti_wave_capacity_lift.version == "v143-anti-wave-capacity-lift" and .packed_runtime_density.version == "v144-packed-runtime-density" and .l2_prefix_contour.version == "v145-l2-prefix-contour" and .l3_to_l2_rerank.version == "v146-l3-bias-to-l2-rerank" and .nonlinear_density_verdict.version == "v147-nonlinear-density-verdict" and .adversarial_density_corpus.version == "v148-adversarial-density-corpus" and .baseline_duel_report.version == "v149-baseline-duel-report" and .margin_baseline_compare.version == "v150-margin-erosion-baseline-compare" and .anti_wave_ablation.version == "v151-anti-wave-ablation" and .fixed_basis_capacity_sweep.version == "v152-fixed-basis-capacity-sweep" and .useful_capacity_score.version == "v153-useful-capacity-score" and .packed_density_hot_loop.version == "v154-packed-density-hot-loop" and .perf_density_mode.version == "v155-perf-density-mode" and .l2_candidate_cache.version == "v156-l2-candidate-cache" and .l3_phase_bias_into_l2.version == "v157-l3-phase-bias-into-l2" and .nonlinear_density_verdict.nonlinear_density_proven == false and .rows[0].records == 16 and .rows[0].state == "DENSITY_STABLE" and .rows[0].wins_over_lexical_baseline == true and .rows[0].wins_over_expanded_baselines == true and .rows[0].lexical_baseline.reversed_false_positive == true and .rows[0].relation_baseline.version == "v132-relation-only-baseline" and .rows[0].naive_vector_baseline.version == "v132-naive-vector-baseline" and .rows[0].phase_lock.version == "v129-phase-lock-metric" and .rows[0].noise_pressure.version == "v130-noise-pressure" and .rows[0].nonlinear_candidate.version == "v131-nonlinear-candidate" and .rows[0].packed_hot_loop_proxy.version == "v133-packed-density-hot-loop-proxy" and .rows[0].perf_counter_plan.version == "v134-perf-counter-plan" and .rows[0].focus_window_experiment.version == "v135-focus-window-experiment" and .rows[0].l2_contour_spec.version == "v136-v137-l2-contour-spec-prototype" and .rows[2].records == 256 and .rows[2].focus_state == "HOT_FOCUS_READY" and .rows[].accuracy == 1' <<<"$memory_density_json" >/dev/null
demo_json="$("$demo" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --text "declaration requires protocols" --format json)"
jq -e '.mode == "llmwave-demo" and .version == "v62-demo-raw-text-adapter" and .state == "PUBLIC_DEMO_READY" and (.weak_spots | length) == 0' <<<"$demo_json" >/dev/null
demo_review_json="$("$demo" "$root/examples/triad-packet.demo-review-empty.json" --input-format json --text "unsupported relation" --format json || true)"
jq -e '.state == "PUBLIC_DEMO_REVIEW" and (.weak_spots | length) >= 1' <<<"$demo_review_json" >/dev/null
demo_raw_json="$("$demo" --from-text "$root/examples/demo-task.raw.txt" --task-id demo-raw --domain certification --text "declaration requires protocols" --format json)"
jq -e '.input_mode == "raw-text" and .raw_adapter.extraction_method == "arrow-triads" and .raw_adapter.triads == 4 and .state == "PUBLIC_DEMO_READY" and .top_pattern == "declaration -> requires -> protocols" and (.weak_spots | length) == 0' <<<"$demo_raw_json" >/dev/null
demo_suite_json="$("$demo" --suite "$root/examples/demo-corpus.json" --format json)"
jq -e '.mode == "llmwave-demo-suite" and .passed == 3 and .total == 3 and .accuracy == 1' <<<"$demo_suite_json" >/dev/null
jq -e '.cases[] | select(.id == "demo-review-empty-memory" and .state == "PUBLIC_DEMO_REVIEW" and (.weak_spots | length) >= 1)' <<<"$demo_suite_json" >/dev/null
trained_budget_json="$("$budget" "$tmp_decode_index" --input-format json)"
jq -e '.pattern_runtime.version == "v40-6m-pattern-runtime-contract" and .pattern_runtime.active_patterns == 1 and .pattern_runtime.fits_pattern_arena == true' <<<"$trained_budget_json" >/dev/null
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
jq -e '.resonance_memory[0].decision == "accept"' "$tmp_feedback" >/dev/null
jq -e '.resonance_memory[0].peak == "certification"' "$tmp_feedback" >/dev/null
jq -e '.resonance_memory[0].waw_status == "WAW_RESONANCE"' "$tmp_feedback" >/dev/null
tmp_positive_index="$(mktemp)"
"$indexer" "$root/examples/triad-packet.interference-search-route-trap.json" "$tmp_feedback" --input-format json --out "$tmp_positive_index" >/dev/null
jq -e '.positive_shortcuts[0].accepted_count == 1' "$tmp_positive_index" >/dev/null
jq -e '.resonance_memory[0].accepted_count == 1' "$tmp_positive_index" >/dev/null
positive_search_json="$("$search" "$tmp_positive_index" --input-format json --query-file "$root/examples/triad-packet.interference-search-route-trap.json" --query-format json --top-k 3)"
jq -e '.constructive_interference.applied == true' <<<"$positive_search_json" >/dev/null
jq -e '.constructive_interference.reinforcements[0].reinforce_peak == "certification"' <<<"$positive_search_json" >/dev/null
jq -e '.resonance_memory.applied == true' <<<"$positive_search_json" >/dev/null
jq -e '.resonance_memory.applications[0].action == "reinforce"' <<<"$positive_search_json" >/dev/null
jq -e '.resonance_memory.applications[0].peak == "certification"' <<<"$positive_search_json" >/dev/null
jq -e '.resonant_field.coherence_memory.resonance_positive_hits == 1' <<<"$positive_search_json" >/dev/null
jq -e '.peaks[0].positive_lane_boost > 0' <<<"$positive_search_json" >/dev/null
jq -e '.peaks[0].resonance_memory_boost > 0' <<<"$positive_search_json" >/dev/null
encode_json="$("$encode" --text "declaration requires protocols" --as-query-packet --input "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --top-k 3)"
jq -e '.mode == "wave-pattern-encoder"' <<<"$encode_json" >/dev/null
jq -e '.encoder_version == "v33-token-pattern-encoder"' <<<"$encode_json" >/dev/null
jq -e '.token_count == 3 and (.field.signature_hex | length) == 256' <<<"$encode_json" >/dev/null
jq -e '.preview_candidate_triads | length >= 2' <<<"$encode_json" >/dev/null
jq -e '.query_packet.candidate_triads | length >= 2' <<<"$encode_json" >/dev/null
jq -e '.packet_similarity.top_similar_triads | length == 3' <<<"$encode_json" >/dev/null
tmp_feedback2="$(mktemp)"
tmp_positive_learned_index="$(mktemp)"
"$feedback" "$tmp_search" --decision accept --note "route trap accepted" --out "$tmp_feedback2" >/dev/null
"$indexer" "$root/examples/triad-packet.interference-search-route-trap.json" "$tmp_feedback" "$tmp_feedback2" --input-format json --out "$tmp_positive_learned_index" >/dev/null
jq -e '.positive_shortcuts[0].accepted_count == 2' "$tmp_positive_learned_index" >/dev/null
jq -e '.resonance_memory[0].accepted_count == 2' "$tmp_positive_learned_index" >/dev/null
learned_positive_json="$("$search" "$tmp_positive_learned_index" --input-format json --query-file "$root/examples/triad-packet.interference-search-route-trap.json" --query-format json --top-k 3)"
jq -e '.constructive_interference.reinforcements[0].effective_boost > 0.08' <<<"$learned_positive_json" >/dev/null
jq -e '.constructive_interference.reinforcements[0].accepted_count == 2' <<<"$learned_positive_json" >/dev/null
jq -e '.resonance_memory.applications[0].accepted_count == 2' <<<"$learned_positive_json" >/dev/null
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
jq -e '.resonance_memory[0].decision == "reject"' "$tmp_negative_feedback" >/dev/null
jq -e '.resonance_memory[0].peak == "customs"' "$tmp_negative_feedback" >/dev/null
"$indexer" "$root/examples/triad-packet.negative-shortcut-base.json" "$tmp_negative_feedback" --input-format json --out "$tmp_negative_index" >/dev/null
indexed_negative_json="$("$search" "$tmp_negative_index" --input-format json --query-file "$root/examples/triad-packet.negative-shortcut-base.json" --query-format json --top-k 3)"
jq -e '.peaks[0].peak == "certification"' <<<"$indexed_negative_json" >/dev/null
jq -e '.destructive_interference.applied == true' <<<"$indexed_negative_json" >/dev/null
jq -e '.resonance_memory.applied == true' <<<"$indexed_negative_json" >/dev/null
jq -e '.resonance_memory.applications[] | select(.action == "suppress" and .peak == "customs")' <<<"$indexed_negative_json" >/dev/null
jq -e '.resonant_field.coherence_memory.resonance_negative_hits == 0' <<<"$indexed_negative_json" >/dev/null
tmp_negative_feedback2="$(mktemp)"
tmp_negative_learned_index="$(mktemp)"
"$feedback" "$tmp_negative_search" --decision reject --note "customs shortcut" --out "$tmp_negative_feedback2" >/dev/null
"$indexer" "$root/examples/triad-packet.negative-shortcut-base.json" "$tmp_negative_feedback" "$tmp_negative_feedback2" --input-format json --out "$tmp_negative_learned_index" >/dev/null
jq -e '.negative_shortcuts[0].rejected_count == 2' "$tmp_negative_learned_index" >/dev/null
jq -e '.resonance_memory[0].rejected_count == 2' "$tmp_negative_learned_index" >/dev/null
learned_negative_json="$("$search" "$tmp_negative_learned_index" --input-format json --query-file "$root/examples/triad-packet.negative-shortcut-base.json" --query-format json --top-k 3)"
jq -e '.destructive_interference.suppressions[0].effective_penalty > 0.18' <<<"$learned_negative_json" >/dev/null
jq -e '.destructive_interference.suppressions[0].rejected_count == 2' <<<"$learned_negative_json" >/dev/null
jq -e '.resonance_memory.applications[] | select(.action == "suppress" and .rejected_count == 2)' <<<"$learned_negative_json" >/dev/null
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
jq -e '.benchmarks.density.iterations == 100 and .benchmarks.density.kernel == "packed_density_probe" and .benchmarks.density.triads_in_memory == 8 and .benchmarks.density.accuracy == 1 and .benchmarks.density.false_positive == 0' <<<"$bench6m_json" >/dev/null
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
bench6m_density_json="$("$bench6m" --mode density --support-build-iterations 10 --triads 16 --format json)"
jq -e '.benchmarks.replay == null and .benchmarks.projection == null and .benchmarks.density.iterations == 10 and .benchmarks.density.triads_in_memory == 16 and .benchmarks.density.accuracy == 1 and .benchmarks.density.false_positive == 0' <<<"$bench6m_density_json" >/dev/null
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
