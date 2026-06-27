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
boundary_economics="$root/nanda-structural-gate/scripts/nanda-boundary-economics"
build_atlas="$root/nanda-structural-gate/scripts/nanda-build-atlas"
guard_action="$root/nanda-structural-gate/scripts/nanda-guard-action"
guard_diff="$root/nanda-structural-gate/scripts/nanda-guard-diff"
profile_guards="$root/nanda-structural-gate/scripts/nanda-profile-guards"
release_gate="$root/nanda-structural-gate/scripts/nanda-release-gate"
skill_readiness="$root/nanda-structural-gate/scripts/nanda-skill-readiness"
field_report="$root/nanda-structural-gate/scripts/nanda-field-report"
field_audit="$root/nanda-structural-gate/scripts/nanda-field-audit"
field_equivalence="$root/nanda-structural-gate/scripts/nanda-field-equivalence"
field_cutover="$root/nanda-structural-gate/scripts/nanda-field-cutover"
field_plate="$root/nanda-structural-gate/scripts/nanda-field-plate"

cargo fmt --check --manifest-path "$root/Cargo.toml"
cargo check --manifest-path "$root/Cargo.toml" >/dev/null
cargo test --manifest-path "$root/Cargo.toml" >/dev/null
version_text="$("$root/target/debug/nanda" --version)"
grep -q '^nanda ' <<<"$version_text"
grep -q 'core_version: sparse-triad-v6.0-llmwave-proof' <<<"$version_text"
grep -q 'nanda_6m:' <<<"$version_text"
skill_readiness_json="$("$skill_readiness" --format json)"
jq -e '.mode == "nanda-skill-readiness" and .verdict == "PUBLIC_V1_READY" and .public_v1_ready == true and (.blockers | length) == 0' <<<"$skill_readiness_json" >/dev/null
jq -e '(.checks[] | select(.check == "field_core_sole_engine" and .status == "PASS")) and (.checks[] | select(.check == "boundary_field_kernel" and .status == "PASS")) and (.checks[] | select(.check == "pattern16_skill_admission" and .status == "PASS")) and (.checks[] | select(.check == "claim_boundaries" and .status == "PASS"))' <<<"$skill_readiness_json" >/dev/null
jq -e '(.checks[] | select(.check == "boundary_field_kernel" and .evidence.kernel_split_files_present == true and .evidence.commands_are_wrappers == true and .evidence.field_records_owner_is_field_core == true and .evidence.field_not_more_permissive == true and .evidence.selected_verdict_present == true and .evidence.diff_kernel_present == true and .evidence.diff_kernel_split_files_present == true and .evidence.diff_kernel_owner_is_field_core == true and .evidence.diff_field_not_more_permissive == true))' <<<"$skill_readiness_json" >/dev/null
jq -e '(.checks[] | select(.check == "pattern16_skill_admission" and .evidence.capacity_profile_checked == "skill-admission" and .evidence.default_capacity_profile == "default" and .evidence.default_profile_mismatch_is_not_blocker == true and .evidence.readiness_uses_skill_admission_profile == true))' <<<"$skill_readiness_json" >/dev/null
jq -e '.claim_boundary.structural_gate_ready == true and .claim_boundary.broad_chat_llm_ready == false and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.hardware_cache_residency_counter_proven == false' <<<"$skill_readiness_json" >/dev/null
big_core_v1_contract_json="$("$llmwave_big" core-v1-contract --format json)"
jq -e '.mode == "llmwave-core-v1-contract" and .verdict == "CORE_V1_CONTRACT_RECORDED_NOT_IMPLEMENTED" and .claim_boundary.core_contract_recorded == true and .claim_boundary.claim_boundary_table_present == true and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false and (.components | length) == 11 and (.required_boundaries | length) == 5' <<<"$big_core_v1_contract_json" >/dev/null
big_core_v1_field_cutover_json="$("$llmwave_big" core-v1-field-cutover --format json)"
jq -e '.mode == "llmwave-core-v1-field-cutover" and .verdict == "CORE_V1_FIELD_OPERATIONS_CUTOVER_RECORDED_NOT_LLM" and .claim_boundary.field_core_as_sole_field_operations_engine == true and .claim_boundary.field_core_as_sole_llmwave_core_engine == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false and (.family_cutovers | length) == 3 and (.operation_contract | length) == 7' <<<"$big_core_v1_field_cutover_json" >/dev/null
jq -e '.unified_field.field_pass.version == "unified-field-pass-v1" and .cognitive_field_engine.field_core_as_llm == false' <<<"$big_core_v1_field_cutover_json" >/dev/null
big_core_v1_memory_writer_json="$("$llmwave_big" core-v1-memory-writer --format json)"
jq -e '.mode == "llmwave-core-v1-memory-writer" and .verdict == "CORE_V1_MEMORY_WRITER_READY_NOT_NONLINEAR_PROOF" and .phase_3_exit_criteria.residual_write_path_active == true and .phase_3_exit_criteria.raw_dictionary_is_not_primary_memory == true and .phase_3_exit_criteria.memory_write_report_present == true and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_core_v1_memory_writer_json" >/dev/null
jq -e '.byte_report.writer_saving_ratio > 0 and .rejected.rejected_duplicate_count > 0 and .rejected.rejected_noise_count > 0 and .unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_core_v1_memory_writer_json" >/dev/null
big_core_v1_nonlinear_proof_json="$("$llmwave_big" core-v1-nonlinear-proof --format json)"
jq -e '.mode == "llmwave-core-v1-nonlinear-proof" and .verdict == "CORE_V1_NONLINEAR_MEMORY_CANDIDATE_BLOCKED" and .claim_boundary.nonlinear_memory_candidate == true and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_core_v1_nonlinear_proof_json" >/dev/null
jq -e '.proof_metrics.bytes_per_useful_fact_falls_at_three_scale_points == true and .proof_metrics.heldout_quality_bound_to_writer == false and .eval_evidence.external_corpus_present == false and (.claim_boundary.blocked_by | index("heldout_quality_not_bound_to_memory_writer")) and .unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_core_v1_nonlinear_proof_json" >/dev/null
big_core_v1_query_wave_json="$("$llmwave_big" core-v1-query-wave --text "Has customs cleared the goods?" --format json)"
jq -e '.mode == "llmwave-core-v1-query-wave" and .verdict == "CORE_V1_QUERY_WAVE_READY_NOT_RETRIEVAL" and .field_state == "QUERY_WAVE_STRUCTURED" and .safe_to_answer == false and .claim_boundary.query_wave_v1_implemented == true and .claim_boundary.retrieval_ready == false and .claim_boundary.answer_generation_ready == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_core_v1_query_wave_json" >/dev/null
jq -e '(.exit_criteria | all(.passed == true)) and (.exit_eval[] | select(.case_id == "role_swap_invoice_actor" and .observed_state == "QUERY_WAVE_REVERSED_VETO" and .passed == true)) and (.exit_eval[] | select(.case_id == "missing_evidence_release" and .observed_state == "QUERY_WAVE_MISSING_EVIDENCE_NO_ANSWER" and .safe_to_answer == false and .passed == true)) and .unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_core_v1_query_wave_json" >/dev/null
big_core_v1_active_retrieval_json="$("$llmwave_big" core-v1-active-retrieval --text "Has customs cleared the goods?" --format json)"
jq -e '.mode == "llmwave-core-v1-active-retrieval" and .verdict == "CORE_V1_ACTIVE_FIELD_RETRIEVAL_READY_NOT_REASONING" and .output.field_state == "FIELD_FOCUSED" and .output.top_peak == "customs-clearance-status" and .output.safe_to_answer == true and .claim_boundary.retrieval_ready == true and .claim_boundary.schema_reasoning_ready == false and .claim_boundary.answer_generation_ready == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_core_v1_active_retrieval_json" >/dev/null
jq -e '.metrics.required_field_states_covered == .metrics.required_field_state_count and (.exit_criteria | all(.passed == true)) and (.eval_cases[] | select(.case_id == "thin_assertion_trap" and .observed_state == "FIELD_THIN" and .safe_to_answer == false and .passed == true)) and (.eval_cases[] | select(.case_id == "contested_invoice_customs" and .observed_state == "FIELD_CONTESTED" and .safe_to_answer == false and .passed == true)) and .unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_core_v1_active_retrieval_json" >/dev/null
big_core_v1_schema_reasoning_json="$("$llmwave_big" core-v1-schema-reasoning --text "Has customs cleared the goods?" --format json)"
jq -e '.mode == "llmwave-core-v1-schema-reasoning" and .verdict == "CORE_V1_SCHEMA_REASONING_READY_NOT_SURFACE" and .answer_plan.answer_state == "MISSING_DEPENDENCY_DECLARATION_PACKET" and .answer_plan.safe_for_surface_generation == true and .claim_boundary.schema_reasoning_ready == true and .claim_boundary.surface_generation_ready == false and .claim_boundary.answer_verifier_ready == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_core_v1_schema_reasoning_json" >/dev/null
jq -e '(.exit_criteria | all(.passed == true)) and (.dependency_chain[] | select(.operator == "missing" and .state == "missing_blocks_answer")) and (.eval_cases[] | select(.case_id == "contradiction_refusal" and .observed_state == "CONTRADICTION_REFUSED_UNSUPPORTED" and .observed_surface_permission == false and .passed == true)) and (.eval_cases[] | select(.case_id == "role_swap_block" and .observed_state == "ROLE_SWAP_BLOCKED" and .observed_surface_permission == false and .passed == true)) and .unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_core_v1_schema_reasoning_json" >/dev/null
big_core_v1_surface_generation_json="$("$llmwave_big" core-v1-surface-generation --text "Has customs cleared the goods?" --format json)"
jq -e '.mode == "llmwave-core-v1-surface-generation" and .verdict == "CORE_V1_SURFACE_GENERATION_READY_NOT_VERIFIED" and .surface.state == "MISSING_EVIDENCE_REFUSAL" and .surface.safe_for_verifier == true and .claim_boundary.evidence_bound_surface_ready == true and .claim_boundary.free_form_generation == false and .claim_boundary.answer_verifier_ready == false and .claim_boundary.final_answer_ready == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_core_v1_surface_generation_json" >/dev/null
jq -e '(.exit_criteria | all(.passed == true)) and (.forbidden_behavior | all(.blocked == true)) and (.surface.evidence_routes | index("customs-clearance-status")) and (.surface.role_bindings[] | select(.role == "forbidden_shortcut")) and .unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_core_v1_surface_generation_json" >/dev/null
big_core_v1_answer_verifier_json="$("$llmwave_big" core-v1-answer-verifier --text "Has customs cleared the goods?" --format json)"
jq -e '.mode == "llmwave-core-v1-answer-verifier" and .verdict == "CORE_V1_ANSWER_VERIFIER_READY_LOCAL_ONLY" and .verifier.decision == "VERIFIED_REFUSAL_READY" and .verifier.answer_state == "LOCAL_FINAL_REFUSAL" and .verifier.safe_to_answer == true and .claim_boundary.verified_refusal_ready == true and .claim_boundary.positive_answer_ready == false and .claim_boundary.feedback_learning_ready == false and .claim_boundary.general_chat_ready == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_core_v1_answer_verifier_json" >/dev/null
jq -e '(.exit_criteria | all(.passed == true)) and (.blocking_rules | all(.active == true)) and (.verifier.blocked_shortcuts | index("invoice_or_payment_implies_customs_release")) and (.eval_cases[] | select(.case_id == "positive_clearance_without_release_evidence" and .observed_decision == "UNSAFE_SURFACE_REJECTED" and .observed_safe_to_answer == false and .passed == true)) and .unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_core_v1_answer_verifier_json" >/dev/null
big_core_v1_feedback_learning_json="$("$llmwave_big" core-v1-feedback-learning --text "Has customs cleared the goods?" --format json)"
jq -e '.mode == "llmwave-core-v1-feedback-learning" and .verdict == "CORE_V1_FEEDBACK_LEARNING_READY_NOT_CONSOLIDATED" and .memory_packet.packet_state == "FEEDBACK_PACKET_APPLIED_TO_NEXT_PASS" and .memory_packet.shortcut_specific == true and .memory_packet.route_kill_switch == false and .next_field_pass.field_changed == true and .next_field_pass.refusal_delta > 0 and .next_field_pass.shortcut_delta < 0 and .claim_boundary.memory_packet_ready == true and .claim_boundary.consolidation_ready == false and .claim_boundary.broad_training_ready == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_core_v1_feedback_learning_json" >/dev/null
jq -e '(.exit_criteria | all(.passed == true)) and (.memory_packet.lanes | length == 2) and (.eval_cases[] | select(.case_id == "reject_positive_shortcut" and .observed_effect == "suppress_shortcut_only" and .passed == true)) and .unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_core_v1_feedback_learning_json" >/dev/null
big_core_v1_consolidation_sleep_json="$("$llmwave_big" core-v1-consolidation-sleep --text "Has customs cleared the goods?" --format json)"
jq -e '.mode == "llmwave-core-v1-consolidation-sleep" and .verdict == "CORE_V1_CONSOLIDATION_SLEEP_READY_NOT_BROAD_EVAL" and .sleep_pass.after_records < .sleep_pass.before_records and .sleep_pass.preserved_negative_lanes > 0 and .sleep_pass.route_kill_switch == false and .post_sleep_field.shortcut_still_suppressed == true and .claim_boundary.consolidation_ready == true and .claim_boundary.broad_eval_ready == false and .claim_boundary.broad_training_ready == false and .claim_boundary.general_chat_ready == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_core_v1_consolidation_sleep_json" >/dev/null
jq -e '(.exit_criteria | all(.passed == true)) and (.consolidated_memory.retained_forms | index("negative_shortcut_lane")) and (.eval_cases[] | select(.case_id == "watch_decay_not_accept" and .passed == true)) and .unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_core_v1_consolidation_sleep_json" >/dev/null
big_core_v1_broad_eval_json="$("$llmwave_big" core-v1-broad-eval --text "Has customs cleared the goods?" --format json)"
jq -e '.mode == "llmwave-core-v1-broad-eval-harness" and .verdict == "CORE_V1_BROAD_EVAL_HARNESS_READY_NOT_LLM" and .suite.failed == 0 and .suite.false_positive_count == 0 and .suite.false_negative_count == 0 and .claim_boundary.local_core_v1_pipeline_ready == true and .claim_boundary.safety_controls_ready == true and .claim_boundary.real_broad_corpus_loaded == false and .claim_boundary.broad_generalization_proven == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_core_v1_broad_eval_json" >/dev/null
jq -e '(.exit_criteria | all(.passed == true)) and (.suite.cases | length == 10) and (.suite.cases[] | select(.case_id == "broad_corpus_claim_blocked" and .observed == "BROAD_CORPUS_MISSING" and .passed == true)) and (.blockers | all(.active == true)) and .unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_core_v1_broad_eval_json" >/dev/null
big_core_v2_contract_json="$("$llmwave_big" core-v2-contract --format json)"
jq -e '.mode == "llmwave-core-v2-contract" and .verdict == "CORE_V2_CONTRACT_READY_NOT_IMPLEMENTED" and (.record_contracts | length) == 5 and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false and .unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_core_v2_contract_json" >/dev/null
big_core_v2_corpus_json="$("$llmwave_big" core-v2-corpus --format json)"
jq -e '.mode == "llmwave-core-v2-corpus" and .verdict == "CORE_V2_CORPUS_ARTIFACT_READY_FIXTURE_ONLY" and .corpus.fact_count == 12 and .corpus.fixture_only == true and .claim_boundary.fixture_corpus_ready == true and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_core_v2_corpus_json" >/dev/null
big_core_v2_heldout_json="$("$llmwave_big" core-v2-heldout --format json)"
jq -e '.mode == "llmwave-core-v2-heldout" and .verdict == "CORE_V2_HELDOUT_READY_FIXTURE_ONLY" and (.heldout | length) == 4 and .leakage_control.exact_heldout_removed == true and .leakage_control.leakage_rate == 0 and .claim_boundary.heldout_suite_ready == true' <<<"$big_core_v2_heldout_json" >/dev/null
big_core_v2_focus_json="$("$llmwave_big" core-v2-focus --format json)"
jq -e '.mode == "llmwave-core-v2-focus" and .verdict == "CORE_V2_FOCUS_READY_FIXTURE_ONLY" and .focus_packet.heldout_removed == 4 and .focus_packet.route_balanced == true and .focus_packet.selected_facts <= 15000 and .claim_boundary.route_balanced_focus_ready == true' <<<"$big_core_v2_focus_json" >/dev/null
big_core_v2_density_json="$("$llmwave_big" core-v2-density --format json)"
jq -e '.mode == "llmwave-core-v2-density" and .verdict == "CORE_V2_DENSITY_CANDIDATE_NOT_PROVEN" and .claim_boundary.density_candidate == true and .economics.beats_linear_at_fixture_scale == false and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_core_v2_density_json" >/dev/null
big_core_v2_run_json="$("$llmwave_big" core-v2-run --text "Has customs cleared the goods?" --format json)"
jq -e '.mode == "llmwave-core-v2-run" and .verdict == "CORE_V2_RUN_LOCAL_ROUTE_READY_NOT_CHAT" and .field_state == "FIELD_FOCUSED" and .answer.answer_state == "LOCAL_EVIDENCE_BOUND_REFUSAL" and .answer.safe_to_answer == true and (.answer.blocked_shortcuts | index("invoice_proves_customs_release")) and .claim_boundary.llm_ready == false' <<<"$big_core_v2_run_json" >/dev/null
big_core_v2_pack_hot_json="$("$llmwave_big" core-v2-pack-hot --format json)"
jq -e '.mode == "llmwave-core-v2-pack-hot" and .verdict == "CORE_V2_HOT_PACKET_READY_NOT_CACHE_ONLY_PROOF" and .hot_packet.fits_budget == true and .hot_packet.json_used_in_hot_scan == false and .hot_packet.cache_only_execution_proven == false and .claim_boundary.hot_packet_ready == true' <<<"$big_core_v2_pack_hot_json" >/dev/null
big_core_v2_claim_gate_json="$("$llmwave_big" core-v2-claim-gate --format json)"
jq -e '.mode == "llmwave-core-v2-claim-gate" and .verdict == "CORE_V2_LOCAL_PIPELINE_READY_NOT_LLM" and .local_pipeline.local_ready == true and (.blockers | all(.active == true)) and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.real_broad_corpus_loaded == false and .claim_boundary.cache_only_execution_proven == false' <<<"$big_core_v2_claim_gate_json" >/dev/null
big_core_v3_plan_json="$("$llmwave_big" core-v3-plan --format json)"
jq -e '.mode == "llmwave-core-v3-plan" and .verdict == "CORE_V3_PLAN_READY" and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false and (.record_contracts | length) == 5' <<<"$big_core_v3_plan_json" >/dev/null
big_core_v3_solution_json="$("$llmwave_big" core-v3-solution-search --goal "confirm customs clearance" --format json)"
jq -e '.mode == "llmwave-core-v3-solution-search" and .verdict == "CORE_V3_SOLUTION_SEARCH_READY_NOT_GENERAL_REASONER" and .solution.solution_state == "SOLUTION_PATH_FOUND_MISSING_EVIDENCE" and .solution.safe_to_answer_steps == true and .solution.final_fact_confirmed == false and (.solution.blocked_shortcuts | index("invoice_proves_customs_release")) and .claim_boundary.llm_ready == false' <<<"$big_core_v3_solution_json" >/dev/null
big_core_v3_pack_json="$("$llmwave_big" core-v3-pack-1m --format json)"
jq -e '.mode == "llmwave-core-v3-pack-1m" and .verdict == "CORE_V3_1M_ACTIVE_PROJECTION_FITS_6M_NOT_LOSSLESS_STORAGE" and .external_artifact.fact_count == 1000000 and .focus_summary.selected_fact_count == 15000 and .budget.fits_6m == true and .claim_boundary.lossless_million_fact_hot_storage == false and .claim_boundary.llm_ready == false' <<<"$big_core_v3_pack_json" >/dev/null
big_core_v3_claim_json="$("$llmwave_big" core-v3-claim-gate --goal "confirm customs clearance" --format json)"
jq -e '.mode == "llmwave-core-v3-claim-gate" and .verdict == "CORE_V3_SOLUTION_AND_1M_PROJECTION_READY_NOT_LLM" and .local_solution_machine_ready == true and .million_active_projection_ready == true and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.cache_only_execution_proven == false' <<<"$big_core_v3_claim_json" >/dev/null
big_contract_json="$("$llmwave_big" contract --format json)"
jq -e '.roadmap_block == "v158-v160"' <<<"$big_contract_json" >/dev/null
jq -e '.unified_field.field_pass.version == "unified-field-pass-v1"' <<<"$big_contract_json" >/dev/null
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
jq -e '.roadmap_block == "v361-v390" and .verdict == "L2_READY"' <<<"$big_l2_json" >/dev/null
jq -e '.candidate_cache.record_bytes == 32 and .candidate_cache.top_token_label == "invoice" and .candidate_cache.margin >= 12' <<<"$big_l2_json" >/dev/null
jq -e '.sync_policy.l2_update == "per_keystroke" and .sync_policy.l3_update == "word_boundary_punctuation_semantic_shift"' <<<"$big_l2_json" >/dev/null
jq -e '.candidate_cache.sample[] | select(.label == "inventory" and .anti_score > 0 and .final_score < 0)' <<<"$big_l2_json" >/dev/null
jq -e '.runtime_field.top_surface == "счете" and .runtime_field.top_family == "счет" and .runtime_field.margin >= 12 and .runtime_field.field_state == "L2_WAVE_RUNTIME_READY_NOT_CHAT"' <<<"$big_l2_json" >/dev/null
jq -e '.runtime_field.candidates[] | select(.surface == "счетчик" and .prefix_resonance > 0 and .anti_wave > 0 and .final_score < 0)' <<<"$big_l2_json" >/dev/null
jq -e '.runtime_field.claim_boundary.hot_loop_uses_json == false and .runtime_field.claim_boundary.hot_loop_uses_heap == false and .runtime_field.claim_boundary.chat_ready == false and .runtime_field.claim_boundary.nonlinear_memory_proven == false' <<<"$big_l2_json" >/dev/null
big_hrr_json="$("$llmwave_big" hrr --format json)"
jq -e '.roadmap_block == "v391-v430" and .verdict == "HRR_BINDING_READY_NOT_NONLINEAR_PROOF"' <<<"$big_hrr_json" >/dev/null
jq -e '.metrics.role_recall == 1 and .metrics.cleanup_top1 == 1 and .metrics.noisy_role_recall == 1 and .metrics.collision_reject_rate == 1 and .metrics.ambiguous_cleanup_rate == 0' <<<"$big_hrr_json" >/dev/null
jq -e '.bindings[] | select(.role == "supplier" and .filler == "Honglu" and .recovered == "Honglu" and .exact == true)' <<<"$big_hrr_json" >/dev/null
jq -e '.collision_eval.trap_role == "supplier" and .collision_eval.expected_filler == "Honglu" and .collision_eval.rejected_filler == "Rustrade" and .collision_eval.rejected == true' <<<"$big_hrr_json" >/dev/null
jq -e '.claim_boundary.hrr_binding_implemented == true and .claim_boundary.cleanup_memory_implemented == true and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_hrr_json" >/dev/null
big_schema_bind_json="$("$llmwave_big" schema-bind --format json)"
jq -e '.roadmap_block == "v431-v455" and .verdict == "L3_SCHEMA_BIND_READY_NOT_LLM"' <<<"$big_schema_bind_json" >/dev/null
jq -e '.schema.schema_id == 101 and .schema.operator_id == 3 and .metrics.schema_role_recall == 1 and .metrics.role_error_rate == 0 and .metrics.role_swap_reject_rate == 1' <<<"$big_schema_bind_json" >/dev/null
jq -e '.recovered_roles[] | select(.role == "subject:supplier" and .expected == "Honglu" and .recovered == "Honglu" and .exact == true)' <<<"$big_schema_bind_json" >/dev/null
jq -e '.recovered_roles[] | select(.role == "object:document" and .expected == "invoice" and .recovered == "invoice" and .exact == true)' <<<"$big_schema_bind_json" >/dev/null
jq -e '.role_swap_trap.wrong_claim == "invoice issues Honglu" and .role_swap_trap.rejected == true and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_schema_bind_json" >/dev/null
big_l2_l3_json="$("$llmwave_big" l2-l3-couple --format json)"
jq -e '.roadmap_block == "v456-v480" and .verdict == "L2_L3_COUPLED_READY_NOT_CHAT"' <<<"$big_l2_l3_json" >/dev/null
jq -e '.l2_probe.raw_top == "inventory" and .l2_probe.coupled_top == "invoice" and .l3_schema.schema_id == 101' <<<"$big_l2_l3_json" >/dev/null
jq -e '.metrics.l2_l3_agreement_rate == 1 and .metrics.role_error_rate == 0 and .metrics.disagreement_reject_rate == 1' <<<"$big_l2_l3_json" >/dev/null
jq -e '.disagreement_trap.l2_preferred == "invoice" and .disagreement_trap.l3_expected_filler == "Honglu" and .disagreement_trap.rejected == true' <<<"$big_l2_l3_json" >/dev/null
jq -e '.claim_boundary.l2_l3_storage_mixed == false and .claim_boundary.chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_l2_l3_json" >/dev/null
big_decode_loop_json="$("$llmwave_big" decode-loop --format json)"
jq -e '.roadmap_block == "v481-v520" and .verdict == "COUPLED_DECODE_LOOP_READY_NOT_CHAT"' <<<"$big_decode_loop_json" >/dev/null
jq -e '.bridge_state == "L2_L3_COUPLED_READY_NOT_CHAT" and .final_sequence == ["Honglu","issues","invoice"]' <<<"$big_decode_loop_json" >/dev/null
jq -e '.metrics.completed_steps == 3 and .metrics.sequence_exact == true and .metrics.role_error_rate == 0 and .metrics.bad_continuation_reject_rate == 1' <<<"$big_decode_loop_json" >/dev/null
jq -e '.accepted_steps[0].raw_top == "invoice" and .accepted_steps[0].accepted == "Honglu" and .accepted_steps[2].raw_top == "inventory" and .accepted_steps[2].accepted == "invoice"' <<<"$big_decode_loop_json" >/dev/null
jq -e '.bad_continuation_trap.trap == "invoice_issues_honglu_role_break" and .bad_continuation_trap.rejected == true and .claim_boundary.fixed_step_records == true and .claim_boundary.chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_decode_loop_json" >/dev/null
big_multi_schema_json="$("$llmwave_big" multi-schema --format json)"
jq -e '.roadmap_block == "v521-v560" and .verdict == "MULTI_SCHEMA_COMPETITION_READY_NOT_CHAT"' <<<"$big_multi_schema_json" >/dev/null
jq -e '.decode_bridge_state == "COUPLED_DECODE_LOOP_READY_NOT_CHAT" and .metrics.active_schema_count == 4 and .metrics.selected_schema_id == 101 and .selected_route.route == "supplier-docs"' <<<"$big_multi_schema_json" >/dev/null
jq -e '.selected_route.sequence == ["Honglu","issues","invoice"] and .metrics.top_margin > 0 and .metrics.schema_selection_error_rate == 0' <<<"$big_multi_schema_json" >/dev/null
jq -e '.route_splice_trap.trap == "route_splice_honglu_pays_invoice" and .route_splice_trap.individually_plausible == true and .route_splice_trap.selected_as_whole_route == false and .route_splice_trap.rejected == true' <<<"$big_multi_schema_json" >/dev/null
jq -e '.claim_boundary.fixed_peak_records == true and .claim_boundary.chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_multi_schema_json" >/dev/null
big_schema_grow_json="$("$llmwave_big" schema-grow --format json)"
jq -e '.roadmap_block == "v561-v620" and .verdict == "SCHEMA_MEMORY_GROWTH_READY_NOT_CHAT"' <<<"$big_schema_grow_json" >/dev/null
jq -e '.competition_bridge_state == "MULTI_SCHEMA_COMPETITION_READY_NOT_CHAT" and .observed_fact_count == 11 and .memory_metrics.promoted_count == 3' <<<"$big_schema_grow_json" >/dev/null
jq -e '.promoted_schemas[] | select(.route == "supplier-docs" and .support_count == 3)' <<<"$big_schema_grow_json" >/dev/null
jq -e '.promoted_schemas[] | select(.route == "buyer-payment" and .support_count == 3)' <<<"$big_schema_grow_json" >/dev/null
jq -e '.promoted_schemas[] | select(.route == "customs-check" and .support_count == 3)' <<<"$big_schema_grow_json" >/dev/null
jq -e '.negative_control.proposed_form == "warehouse signs invoice" and .negative_control.rejected == true and .memory_metrics.false_promotion_rate == 0 and .claim_boundary.fixed_learned_schema_records == true and .claim_boundary.chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_schema_grow_json" >/dev/null
big_surface_generate_json="$("$llmwave_big" surface-generate --format json)"
jq -e '.roadmap_block == "v621-v700" and .verdict == "OPEN_SURFACE_GENERATION_READY_NOT_CHAT"' <<<"$big_surface_generate_json" >/dev/null
jq -e '.schema_growth_bridge_state == "SCHEMA_MEMORY_GROWTH_READY_NOT_CHAT" and .selected_schema.route == "supplier-docs"' <<<"$big_surface_generate_json" >/dev/null
jq -e '.materialized_surface == "Honglu issued invoice PI-03 to Rustrade" and .generation_metrics.step_count == 6 and .generation_metrics.exact_surface == true' <<<"$big_surface_generate_json" >/dev/null
jq -e '.trap.proposed_surface == "Honglu paid invoice PI-03 to Rustrade" and .trap.rejected == true and .generation_metrics.trap_reject_rate == 1' <<<"$big_surface_generate_json" >/dev/null
jq -e '.claim_boundary.fixed_surface_step_records == true and .claim_boundary.free_form_chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_surface_generate_json" >/dev/null
big_reason_field_json="$("$llmwave_big" reason-field --format json)"
jq -e '.roadmap_block == "v701-v780" and .verdict == "MULTI_STEP_REASONING_FIELD_READY_NOT_CHAT"' <<<"$big_reason_field_json" >/dev/null
jq -e '.surface_bridge_state == "OPEN_SURFACE_GENERATION_READY_NOT_CHAT" and .premise_surface == "Honglu issued invoice PI-03 to Rustrade"' <<<"$big_reason_field_json" >/dev/null
jq -e '.metrics.hop_count == 3 and .metrics.chain_exact == true and .metrics.contradiction_rate == 0 and .metrics.missing_evidence_reject_rate == 1' <<<"$big_reason_field_json" >/dev/null
jq -e '.inferred_state | index("payment_should_follow_invoice") and index("customs_check_needs_declaration_packet")' <<<"$big_reason_field_json" >/dev/null
jq -e '.trap.proposed_inference == "customs cleared goods" and .trap.rejected == true and .claim_boundary.fixed_reasoning_hop_records == true and .claim_boundary.broad_reasoning_proven == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_reason_field_json" >/dev/null
big_dialogue_state_json="$("$llmwave_big" dialogue-state --format json)"
jq -e '.roadmap_block == "v781-v860" and .verdict == "DIALOGUE_STATE_READY_NOT_CHAT"' <<<"$big_dialogue_state_json" >/dev/null
jq -e '.reasoning_bridge_state == "MULTI_STEP_REASONING_FIELD_READY_NOT_CHAT" and .answer_state == "WATCH_UNSUPPORTED_CLEARANCE"' <<<"$big_dialogue_state_json" >/dev/null
jq -e '.constrained_answer | contains("Not proven") and contains("declaration evidence")' <<<"$big_dialogue_state_json" >/dev/null
jq -e '.trap.unsafe_answer == "Yes, customs cleared the goods." and .trap.rejected == true and .metrics.unsupported_answer_reject_rate == 1' <<<"$big_dialogue_state_json" >/dev/null
jq -e '.claim_boundary.fixed_dialogue_turn_records == true and .claim_boundary.multi_turn_chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_dialogue_state_json" >/dev/null
big_mini_chat_eval_json="$("$llmwave_big" mini-chat-eval --format json)"
jq -e '.roadmap_block == "v861-v950" and .verdict == "MINI_CHAT_EVAL_PASS_NOT_GENERAL_LLM"' <<<"$big_mini_chat_eval_json" >/dev/null
jq -e '.dialogue_bridge_state == "DIALOGUE_STATE_READY_NOT_CHAT" and .metrics.case_count == 5 and .metrics.passed_cases == 5 and .metrics.failed_cases == 0' <<<"$big_mini_chat_eval_json" >/dev/null
jq -e '.metrics.grounded_answer_rate == 1 and .metrics.unsupported_reject_rate == 1 and .metrics.route_splice_reject_rate == 1 and .metrics.surface_exact_rate == 1' <<<"$big_mini_chat_eval_json" >/dev/null
jq -e '([.eval_cases[].case_id] | index("grounded_clearance_answer") and index("unsupported_clearance") and index("route_splice_surface") and index("one_off_schema_noise") and index("exact_constrained_surface"))' <<<"$big_mini_chat_eval_json" >/dev/null
jq -e '.claim_boundary.fixed_eval_case_records == true and .claim_boundary.full_llm_ready == false and .claim_boundary.multi_turn_chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_mini_chat_eval_json" >/dev/null
big_query_wave_json="$("$llmwave_big" query-wave --text "Has customs cleared the goods?" --format json)"
jq -e '.roadmap_block == "v951-v1000" and .verdict == "QUERY_WAVE_READY_NOT_FIELD_MATURE" and .cognitive_field_cutover.old_verdict == "QUERY_WAVE_READY_NOT_FIELD_MATURE" and .cognitive_field_cutover.selected_verdict == "WATCH" and .cognitive_field_cutover.top_level_domain_contract_preserved == true' <<<"$big_query_wave_json" >/dev/null
jq -e '.unified_field.family == "cognitive" and .unified_field.compute_probe.version == "unified-field-compute-v1" and .unified_field.claim_boundary.not_llm_ready == true' <<<"$big_query_wave_json" >/dev/null
jq -e '.unified_field.field_pass.version == "unified-field-pass-v1" and .unified_field.field_pass.family == "cognitive" and .unified_field.field_pass.safe_to_answer == false' <<<"$big_query_wave_json" >/dev/null
jq -e '.field_runtime.version == "unified-field-runtime-v1" and .field_runtime.mode == "cognitive-dual-run" and .field_runtime.cutover_ready == true and .field_runtime.field_safe_to_answer == false' <<<"$big_query_wave_json" >/dev/null
jq -e '.cognitive_field_engine.version == "cognitive-field-engine-guard-v1" and .cognitive_field_engine.field_participates == true and .cognitive_field_engine.candidate_allowed == true and .cognitive_field_engine.selected_engine == "field-core-cognitive-cutover" and .cognitive_field_engine.cutover_applied == true and .cognitive_field_engine.top_level_behavior_changed == false and .cognitive_field_engine.field_core_as_semantic_engine == true and .cognitive_field_engine.field_core_as_sole_engine == true and .cognitive_field_engine.field_core_as_cognitive_sole_engine == true and .cognitive_field_engine.field_core_as_chat_engine == false and .cognitive_field_engine.field_core_as_llm == false and (.cognitive_field_engine.cutover_blocked_reason | length == 0) and .cognitive_field_cutover.applied == true' <<<"$big_query_wave_json" >/dev/null
jq -e '.top_route_hint == "customs-clearance-status" and .question_polarity == "question_status" and .record.l3_schema_hint_id == 203' <<<"$big_query_wave_json" >/dev/null
jq -e '.metrics.paraphrase_route_recall == 1 and .metrics.role_hint_accuracy == 1 and .metrics.operator_hint_accuracy == 1 and .metrics.assertion_reject_rate == 1' <<<"$big_query_wave_json" >/dev/null
jq -e '([.paraphrase_eval[].case_id] | index("en_has_cleared") and index("en_is_cleared") and index("ru_released") and index("assertion_trap"))' <<<"$big_query_wave_json" >/dev/null
jq -e '.claim_boundary.fixed_query_wave_records == true and .claim_boundary.full_field_mature == false and .claim_boundary.chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_query_wave_json" >/dev/null
big_multi_peak_json="$("$llmwave_big" multi-peak-field --text "Has customs cleared the goods?" --format json)"
jq -e '.roadmap_block == "v1001-v1060" and .verdict == "MULTI_PEAK_FIELD_READY_NOT_ANSWER"' <<<"$big_multi_peak_json" >/dev/null
jq -e '.unified_field.family == "cognitive" and .unified_field.compute_probe.version == "unified-field-compute-v1" and .unified_field.claim_boundary.not_llm_ready == true' <<<"$big_multi_peak_json" >/dev/null
jq -e '.query_wave_state == "QUERY_WAVE_READY_NOT_FIELD_MATURE" and .field_state == "STABLE_PEAK" and .top_peak.route == "customs-clearance-status"' <<<"$big_multi_peak_json" >/dev/null
jq -e '.metrics.stable_peak_accuracy == 1 and .metrics.contested_detection_rate == 1 and .metrics.no_answer_detection_rate == 1 and .metrics.route_leakage_reject_rate == 1' <<<"$big_multi_peak_json" >/dev/null
jq -e '([.eval_cases[].expected_state] | index("STABLE_PEAK") and index("CONTESTED") and index("NO_ANSWER") and index("REJECTED"))' <<<"$big_multi_peak_json" >/dev/null
jq -e '.claim_boundary.fixed_peak_records == true and .claim_boundary.safe_to_answer == false and .claim_boundary.full_field_mature == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_multi_peak_json" >/dev/null
big_lens_scan_json="$("$llmwave_big" lens-scan --text "Has customs cleared the goods?" --format json)"
jq -e '.roadmap_block == "v1061-v1140" and .verdict == "LENS_SCAN_READY_NOT_ANSWER"' <<<"$big_lens_scan_json" >/dev/null
jq -e '.unified_field.family == "cognitive" and .unified_field.compute_probe.version == "unified-field-compute-v1" and .unified_field.claim_boundary.not_llm_ready == true' <<<"$big_lens_scan_json" >/dev/null
jq -e '.field_bridge_state == "STABLE_PEAK" and .top_route == "customs-clearance-status" and .answer_decision == "ANSWER_BLOCKED_BY_LENSES"' <<<"$big_lens_scan_json" >/dev/null
jq -e '.metrics.role_lens_pass_rate == 1 and .metrics.evidence_block_rate == 1 and .metrics.answer_block_rate == 1 and .metrics.lens_agreement_rate > 0.5' <<<"$big_lens_scan_json" >/dev/null
jq -e '([.lenses[].lens] | index("role") and index("evidence") and index("temporal") and index("causal") and index("contradiction") and index("surface") and index("answer"))' <<<"$big_lens_scan_json" >/dev/null
jq -e '.field_core_admission.version == "unified-field-pass-v1" and .field_core_admission.family == "cognitive" and .field_core_admission.anti_wave_count == 3 and .field_core_admission.safe_to_answer == false and .field_core_admission.claim_boundary.not_llm_ready == true' <<<"$big_lens_scan_json" >/dev/null
jq -e '.claim_boundary.fixed_lens_records == true and .claim_boundary.safe_to_answer == false and .claim_boundary.chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_lens_scan_json" >/dev/null
big_mature_anti_wave_json="$("$llmwave_big" mature-anti-wave --text "Has customs cleared the goods?" --format json)"
jq -e '.roadmap_block == "v1141-v1210" and .verdict == "MATURE_ANTI_WAVE_READY_NOT_ANSWER"' <<<"$big_mature_anti_wave_json" >/dev/null
jq -e '.unified_field.family == "cognitive" and .unified_field.compute_probe.version == "unified-field-compute-v1" and .unified_field.claim_boundary.not_llm_ready == true' <<<"$big_mature_anti_wave_json" >/dev/null
jq -e '.lens_bridge_verdict == "LENS_SCAN_READY_NOT_ANSWER" and .field_after_anti.anti_field_state == "SUPPRESSED_UNSUPPORTED_ANSWER"' <<<"$big_mature_anti_wave_json" >/dev/null
jq -e '.metrics.lane_count == 3 and .metrics.evidence_lane_rate == 1 and .metrics.causal_lane_rate == 1 and .metrics.answer_lane_rate == 1' <<<"$big_mature_anti_wave_json" >/dev/null
jq -e '.field_core_admission.version == "unified-field-pass-v1" and .field_core_admission.family == "cognitive" and .field_core_admission.anti_wave_count == 3 and .field_core_admission.safe_to_answer == false and .field_core_admission.claim_boundary.not_nonlinear_memory_proof == true' <<<"$big_mature_anti_wave_json" >/dev/null
jq -e '.claim_boundary.fixed_anti_lane_records == true and .claim_boundary.local_suppression_only == true and .claim_boundary.safe_to_answer == false and .claim_boundary.chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_mature_anti_wave_json" >/dev/null
big_evidence_missing_json="$("$llmwave_big" evidence-proof --text "Has customs cleared the goods?" --evidence-mode missing --format json)"
jq -e '.roadmap_block == "v1211-v1280" and .verdict == "EVIDENCE_PROOF_READY_NOT_ANSWER"' <<<"$big_evidence_missing_json" >/dev/null
jq -e '.unified_field.family == "cognitive" and .unified_field.compute_probe.version == "unified-field-compute-v1" and .unified_field.claim_boundary.not_llm_ready == true' <<<"$big_evidence_missing_json" >/dev/null
jq -e '.proof_state == "EVIDENCE_MISSING" and .answer_permission == "ANSWER_BLOCKED_BY_EVIDENCE"' <<<"$big_evidence_missing_json" >/dev/null
jq -e '.claim_boundary.fixed_evidence_proof_records == true and .claim_boundary.local_answer_permission == false and .claim_boundary.safe_to_answer == false and .claim_boundary.chat_ready == false' <<<"$big_evidence_missing_json" >/dev/null
big_evidence_bound_json="$("$llmwave_big" evidence-proof --text "Has customs cleared the goods?" --evidence-mode release-confirmed --format json)"
jq -e '.roadmap_block == "v1211-v1280" and .verdict == "EVIDENCE_PROOF_LOCAL_ANSWER_CANDIDATE"' <<<"$big_evidence_bound_json" >/dev/null
jq -e '.proof_state == "EVIDENCE_BOUND" and .answer_permission == "LOCAL_ANSWER_PERMISSION" and .negative_control.passed == true' <<<"$big_evidence_bound_json" >/dev/null
jq -e '.claim_boundary.fixed_evidence_proof_records == true and .claim_boundary.local_answer_permission == true and .claim_boundary.chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_evidence_bound_json" >/dev/null
big_answer_missing_json="$("$llmwave_big" answer-surface --text "Has customs cleared the goods?" --evidence-mode missing --format json)"
jq -e '.roadmap_block == "v1281-v1350" and .verdict == "ANSWER_SURFACE_NOT_PROVEN"' <<<"$big_answer_missing_json" >/dev/null
jq -e '.answer_state == "NOT_PROVEN_ANSWER" and (.answer_text | contains("Not proven"))' <<<"$big_answer_missing_json" >/dev/null
jq -e '.claim_boundary.fixed_answer_surface_records == true and .claim_boundary.free_form_generation == false and .claim_boundary.chat_ready == false' <<<"$big_answer_missing_json" >/dev/null
big_answer_bound_json="$("$llmwave_big" answer-surface --text "Has customs cleared the goods?" --evidence-mode release-confirmed --format json)"
jq -e '.roadmap_block == "v1281-v1350" and .verdict == "ANSWER_SURFACE_LOCAL_CANDIDATE"' <<<"$big_answer_bound_json" >/dev/null
jq -e '.answer_state == "LOCAL_EVIDENCE_BOUND_ANSWER" and (.answer_text | contains("evidence ref 7001"))' <<<"$big_answer_bound_json" >/dev/null
jq -e '.metrics.constrained_template_rate == 1 and .metrics.unsupported_confirmation_rate == 0 and .claim_boundary.free_form_generation == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_answer_bound_json" >/dev/null
big_feedback_accept_json="$("$llmwave_big" field-feedback --text "Has customs cleared the goods?" --evidence-mode release-confirmed --decision accept --format json)"
jq -e '.roadmap_block == "v1351-v1420" and .verdict == "FIELD_FEEDBACK_REINFORCED"' <<<"$big_feedback_accept_json" >/dev/null
jq -e '.feedback_state == "FEEDBACK_ACCEPTED" and .memory_effect == "reinforce_evidence_bound_route"' <<<"$big_feedback_accept_json" >/dev/null
jq -e '.claim_boundary.fixed_field_feedback_records == true and .claim_boundary.local_memory_update == true and .claim_boundary.persistent_training_done == false' <<<"$big_feedback_accept_json" >/dev/null
big_feedback_reject_json="$("$llmwave_big" field-feedback --text "Has customs cleared the goods?" --evidence-mode release-confirmed --decision reject --format json)"
jq -e '.roadmap_block == "v1351-v1420" and .verdict == "FIELD_FEEDBACK_SUPPRESSED"' <<<"$big_feedback_reject_json" >/dev/null
jq -e '.feedback_state == "FEEDBACK_REJECTED" and .memory_effect == "write_local_anti_memory"' <<<"$big_feedback_reject_json" >/dev/null
jq -e '.metrics.reject_suppression_rate == 1 and .claim_boundary.chat_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_feedback_reject_json" >/dev/null
big_feedback_memory_json="$("$llmwave_big" feedback-memory --text "Has customs cleared the goods?" --evidence-mode release-confirmed --decision accept --format json)"
jq -e '.roadmap_block == "v1421-v1480" and .verdict == "FEEDBACK_MEMORY_READY" and .metrics.record_count == 1' <<<"$big_feedback_memory_json" >/dev/null
jq -e '.claim_boundary.fixed_applied_memory_records == true and .claim_boundary.can_feed_next_field_pass == true and .claim_boundary.persistent_training_done == false' <<<"$big_feedback_memory_json" >/dev/null
big_feedback_field_json="$("$llmwave_big" feedback-aware-field --text "Has customs cleared the goods?" --memory-mode accept --format json)"
jq -e '.roadmap_block == "v1481-v1540" and .verdict == "FEEDBACK_AWARE_FIELD_REINFORCED"' <<<"$big_feedback_field_json" >/dev/null
jq -e '.unified_field.family == "cognitive" and .unified_field.compute_probe.version == "unified-field-compute-v1" and .unified_field.claim_boundary.not_llm_ready == true' <<<"$big_feedback_field_json" >/dev/null
jq -e '.metrics.adjusted_top_score > .metrics.baseline_top_score and .claim_boundary.feedback_applied_to_field == true and .claim_boundary.safe_to_answer == false' <<<"$big_feedback_field_json" >/dev/null
big_anti_memory_json="$("$llmwave_big" applied-anti-memory --format json)"
jq -e '.roadmap_block == "v1541-v1600" and .verdict == "APPLIED_ANTI_MEMORY_READY"' <<<"$big_anti_memory_json" >/dev/null
jq -e '.claim_boundary.suppresses_false_route == true and .claim_boundary.preserves_true_route == true and .claim_boundary.global_memory_deleted == false' <<<"$big_anti_memory_json" >/dev/null
tmp_big_memory="$(mktemp)"
big_memory_store_json="$("$llmwave_big" memory-store --path "$tmp_big_memory" --action apply --decision accept --format json)"
jq -e '.roadmap_block == "v1601-v1660" and .verdict == "PERSISTENT_MEMORY_STORE_READY" and .store.record_count == 1' <<<"$big_memory_store_json" >/dev/null
jq empty "$tmp_big_memory"
rm -f "$tmp_big_memory"
big_learning_eval_json="$("$llmwave_big" learning-eval --format json)"
jq -e '.roadmap_block == "v1661-v1720" and .verdict == "LEARNING_EVAL_PASS_FIXTURE"' <<<"$big_learning_eval_json" >/dev/null
jq -e '.metrics.accepted_route_lift > 0 and .metrics.rejected_route_suppression > 0 and .metrics.regression_rate == 0' <<<"$big_learning_eval_json" >/dev/null
big_memory_consolidate_json="$("$llmwave_big" memory-consolidate --format json)"
jq -e '.roadmap_block == "v1721-v1780" and .verdict == "MEMORY_CONSOLIDATION_READY"' <<<"$big_memory_consolidate_json" >/dev/null
jq -e '.records_after < .records_before and .memory_bytes_after < .memory_bytes_before and .claim_boundary.duplicate_growth_controlled == true' <<<"$big_memory_consolidate_json" >/dev/null
big_runtime_json="$("$llmwave_big" run --evidence-mode release-confirmed --decision accept --format json)"
jq -e '.roadmap_block == "v1781-v1840" and .verdict == "RUNTIME_PIPELINE_READY_FIXTURE" and .final_state == "LOCAL_EVIDENCE_BOUND_PIPELINE"' <<<"$big_runtime_json" >/dev/null
jq -e '.claim_boundary.full_pipeline_implemented == true and .claim_boundary.free_form_generation == false and .claim_boundary.chat_ready == false' <<<"$big_runtime_json" >/dev/null
big_core_eval_json="$("$llmwave_big" core-eval --format json)"
jq -e '.roadmap_block == "v1841-v1900" and .verdict == "CORE_RUNTIME_READY_FIXTURE"' <<<"$big_core_eval_json" >/dev/null
jq -e '.criteria.feedback_applied_to_next_run == true and .criteria.memory_persists_across_process_restart == true and .claim_boundary.core_runtime_ready == true and .claim_boundary.full_llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_core_eval_json" >/dev/null
big_structural_capacity_json="$("$llmwave_big" structural-capacity --format json)"
jq -e '.mode == "llmwave-big-structural-capacity-1024-pattern16" and .verdict == "STRUCTURAL_CAPACITY_1024_PATTERN16_BASELINE_BEATEN" and .workload.patterns == 1024 and .workload.pattern_shape == "Pattern16 macro-cell" and .workload.edges_per_pattern == 16 and .workload.active_facts == 16384' <<<"$big_structural_capacity_json" >/dev/null
jq -e '.workload.fixed_pattern_count == true and .workload.fixed_pattern_shape == true and .workload.smaller_pattern_modes_available == false and .workload.smaller_pattern_shapes_available == false' <<<"$big_structural_capacity_json" >/dev/null
jq -e '.gates.fixed_1024_only == true and .gates.pattern16_macro_cell == true and .gates.missing_edge_rejection == true and .gates.final_gate_passed == true and .metrics.false_accept_rate == 0 and .metrics.false_negative_rate == 0 and .claim_boundary.structural_capacity_1024_ready == true' <<<"$big_structural_capacity_json" >/dev/null
jq -e '.gates.field_core_lens_admission == true and .lens_admission.uses_existing_field_core_lens == true and .lens_admission.uses_existing_field_core_anti_wave == true and .lens_admission.field_pass_peak_target == "pattern16-structural-capacity" and .lens_admission.field_pass_verdict == "WATCH" and .lens_admission.field_pass_safe_to_answer == false and .lens_admission.claim_boundary_preserved == true' <<<"$big_structural_capacity_json" >/dev/null
jq -e '.claim_boundary.broad_chat_llm_ready == false and .claim_boundary.global_nonlinear_memory_proven == false and .claim_boundary.hardware_cache_residency_counter_proven == false' <<<"$big_structural_capacity_json" >/dev/null
big_structural_capacity_admission_json="$("$llmwave_big" structural-capacity --noise-profile skill-admission --format json)"
jq -e '.workload.noise_profile == "skill-admission" and .workload.requested_seeds == 2 and .workload.requested_noise_edges_per_noisy_case == 4 and .workload.seeds == 8 and .workload.noise_edges_per_noisy_case == 16' <<<"$big_structural_capacity_admission_json" >/dev/null
jq -e '.gates.skill_admission_noise_profile == true and .gates.skill_admission_noise_pressure == true and .gates.single_peak_under_noise == true and .gates.field_core_lens_admission == true and .gates.anti_wave_traps_reject_false_peaks == true and .gates.final_gate_passed == true' <<<"$big_structural_capacity_admission_json" >/dev/null
jq -e '.lens_admission.accepted_for_skill_admission == true and .lens_admission.single_peak_confirmed == true and .lens_admission.anti_wave_local_false_peak_blockers == true' <<<"$big_structural_capacity_admission_json" >/dev/null
big_readiness_json="$("$llmwave_big" readiness-ladder --format json)"
jq -e '.mode == "llmwave-big-readiness-ladder" and .current_level == 3 and .current_state == "CONSTRAINED_FIELD_ENGINE_READY_NOT_GENERAL_LLM"' <<<"$big_readiness_json" >/dev/null
jq -e '.claim_boundary.field_core_as_sole_engine == true and .claim_boundary.fixture_reasoning_ready == true and .claim_boundary.artifact_grounded_qa_ready == true and .claim_boundary.scripted_hot_multi_turn_ready == true and .claim_boundary.small_domain_llmwave_ready == true and .claim_boundary.scale_amortized_nonlinear_memory_ready == true and .claim_boundary.broad_chat_llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_readiness_json" >/dev/null
jq -e '.llmwave_migration.verdict == "LLMWAVE_FIELD_MIGRATION_READY_NOT_GENERAL_LLM" and .llmwave_migration.field_core_cutover_ready == true and .llmwave_migration.pattern16_macro_cell_ready == true and .llmwave_migration.feedback_changes_next_field == true and .llmwave_migration.final_claims_blocked == true' <<<"$big_readiness_json" >/dev/null
jq -e '.claim_boundary.llmwave_migration_ready == true and .claim_boundary.llmwave_field_core_backed == true and .claim_boundary.pattern16_macro_cell_ready == true and .claim_boundary.feedback_changes_next_field == true and .claim_boundary.broad_chat_llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_readiness_json" >/dev/null
jq -e '([.levels[].name] | index("scale-amortized nonlinear memory"))' <<<"$big_readiness_json" >/dev/null
big_field_claim_json="$("$llmwave_big" claim-gate --claim field-core-sole-engine --format json)"
jq -e '.mode == "llmwave-big-claim-gate" and .claim == "field-core-sole-engine" and .verdict == "CLAIM_ALLOWED" and .allowed == true' <<<"$big_field_claim_json" >/dev/null
jq -e '(.evidence | index("nanda-field-audit sole_engine_contract")) and (.evidence | index("Pattern16 admission field pass through field_core")) and (.missing_evidence | length == 0)' <<<"$big_field_claim_json" >/dev/null
big_active65k_claim_json="$("$llmwave_big" claim-gate --claim active-65k-runtime --format json)"
jq -e '.mode == "llmwave-big-claim-gate" and .claim == "active-65k-runtime" and .verdict == "CLAIM_ALLOWED_LOCAL_RUNTIME_ONLY" and .allowed == true and .claim_boundary.active_65k_runtime_ready == true and .claim_boundary.broad_chat_llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_active65k_claim_json" >/dev/null
tmp_linux_atlas="$(mktemp -d)"
big_linux_atlas_json="$("$llmwave_big" linux-atlas-build --out-dir "$tmp_linux_atlas" --pack-kind base --max-facts 64 --format json)"
jq -e '.mode == "llmwave-big-linux-atlas-build" and (.verdict | startswith("LINUX_ATLAS_")) and .pack_kind == "base" and .artifact.fact_count > 0 and .artifact.previous_fact_count == 0 and .artifact.append_only == true and .artifact.delta_ready == true' <<<"$big_linux_atlas_json" >/dev/null
jq -e '.claim_boundary.append_only_memory_ready == true and .claim_boundary.active_65k_pack_ready == false and .claim_boundary.exposure_layer_ready == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_linux_atlas_json" >/dev/null
test -s "$(jq -r '.outputs.facts_path' <<<"$big_linux_atlas_json")"
test -s "$(jq -r '.outputs.manifest_path' <<<"$big_linux_atlas_json")"
test -s "$(jq -r '.outputs.fact_id_index_path' <<<"$big_linux_atlas_json")"
test -s "$(jq -r '.outputs.pack_log_path' <<<"$big_linux_atlas_json")"
big_linux_atlas_delta_json="$("$llmwave_big" linux-atlas-build --out-dir "$tmp_linux_atlas" --pack-kind delta --max-facts 128 --format json)"
jq -e '.pack_kind == "delta" and .artifact.fact_count > 0 and .artifact.previous_fact_count > 0 and .artifact.delta_new_facts == .artifact.fact_count and .artifact.append_only == true and .claim_boundary.append_only_memory_ready == true' <<<"$big_linux_atlas_delta_json" >/dev/null
test -s "$(jq -r '.outputs.facts_path' <<<"$big_linux_atlas_delta_json")"
tmp_linux_active="$(mktemp -d)"
mkdir -p "$tmp_linux_active/facts"
tmp_linux_active_facts="$tmp_linux_active/facts/base.jsonl"
cat >"$tmp_linux_active_facts" <<JSONL
{"fact_id":"linux.test.bash","layer":"linux-knowledge","domain":"apt-command-index","route":"linux.apt.command.provider","subject":"bash","relation":"provided by package","object":"bash","polarity":"positive","confidence":90,"evidence":{"source_kind":"fixture","path":"fixture","line":1,"extractor":"fixture"}}
{"fact_id":"linux.test.checkbashisms.trap","layer":"linux-knowledge","domain":"apt-command-index","route":"linux.apt.command.package-command","subject":"checkbashisms","relation":"provides command","object":"checkbashisms","polarity":"positive","confidence":95,"evidence":{"source_kind":"fixture","path":"fixture","line":2,"extractor":"fixture"}}
{"fact_id":"linux.test.systemctl","layer":"linux-knowledge","domain":"package-file-list","route":"linux.package.binary","subject":"systemd","relation":"provides binary","object":"/usr/bin/systemctl","polarity":"positive","confidence":90,"evidence":{"source_kind":"fixture","path":"fixture","line":3,"extractor":"fixture"}}
{"fact_id":"linux.test.which.trap","layer":"linux-knowledge","domain":"apt-command-index","route":"linux.apt.command.package-command","subject":"which.debianutils","relation":"provides command","object":"which.debianutils","polarity":"positive","confidence":95,"evidence":{"source_kind":"fixture","path":"fixture","line":4,"extractor":"fixture"}}
{"fact_id":"linux.test.systemd","layer":"linux-knowledge","domain":"systemd","route":"linux.systemd.exec","subject":"ssh.service","relation":"execstart","object":"/usr/sbin/sshd","polarity":"positive","confidence":88,"evidence":{"source_kind":"fixture","path":"fixture","line":5,"extractor":"fixture"}}
{"fact_id":"linux.test.boundary.package","layer":"negative-boundary","domain":"linux-boundary","route":"linux.boundary.package","subject":"package installed","relation":"does not prove","object":"binary is running","polarity":"negative","confidence":95,"evidence":{"source_kind":"fixture","path":"fixture","line":6,"extractor":"fixture"}}
{"fact_id":"linux.test.boundary.socket","layer":"negative-boundary","domain":"linux-boundary","route":"linux.boundary.socket","subject":"port listening","relation":"does not prove","object":"firewall allows external packets","polarity":"negative","confidence":95,"evidence":{"source_kind":"fixture","path":"fixture","line":7,"extractor":"fixture"}}
JSONL
printf '{"facts_path":"%s","fact_count":7}\n' "$tmp_linux_active_facts" >"$tmp_linux_active/packs.jsonl"
big_linux_active_json="$("$llmwave_big" linux-active-field --atlas-dir "$tmp_linux_active" --max-active-facts 7 --query "which package provides command bash" --format json)"
jq -e '.mode == "llmwave-big-linux-active-field" and .verdict == "LINUX_ACTIVE_FIELD_READY_NOT_LLM" and .active_pack.selected_facts == 7 and .active_pack.selected_route_count == 6 and .memory_budget.fits_6m_hot_projection == true' <<<"$big_linux_active_json" >/dev/null
jq -e '.claim_boundary.active_field_ready == true and .claim_boundary.query_probe_ready == true and .claim_boundary.binary_hot_packet_ready == false and .claim_boundary.exposure_layer_ready == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_linux_active_json" >/dev/null
jq -e '.probe_results[] | select(.query == "which package provides command bash" and .state == "FOCUSED" and .top_facts[0].object == "bash")' <<<"$big_linux_active_json" >/dev/null
jq -e '.probe_results[] | select(.query == "package installed does not prove binary is running" and .state == "BOUNDARY_FOCUSED")' <<<"$big_linux_active_json" >/dev/null
tmp_linux_hot="$tmp_linux_active/linux-active.laf"
big_linux_pack_hot_json="$("$llmwave_big" linux-pack-hot --atlas-dir "$tmp_linux_active" --max-active-facts 7 --out "$tmp_linux_hot" --format json)"
jq -e '.mode == "llmwave-big-linux-pack-hot" and .verdict == "LINUX_HOT_PACKET_READY_NOT_CACHE_ONLY_PROOF" and .source.selected_facts == 7 and .packet.fixed_record_count == 7 and .packet.fixed_records_fit_6m == true and .packet.json_used_in_hot_scan == false' <<<"$big_linux_pack_hot_json" >/dev/null
jq -e '.claim_boundary.binary_hot_packet_written == true and .claim_boundary.fixed_records_scan_ready == true and .claim_boundary.cache_only_execution_proven == false and .claim_boundary.llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_linux_pack_hot_json" >/dev/null
jq -e '.schema_residual_memory.schema_key == "route+relation+polarity" and .schema_residual_memory.nonlinear_memory_proven == false' <<<"$big_linux_pack_hot_json" >/dev/null
test -s "$tmp_linux_hot"
big_linux_ask_hot_json="$("$llmwave_big" linux-ask-hot --hot-pack "$tmp_linux_hot" --query "which package provides command bash" --top-k 3 --format json)"
jq -e '.mode == "llmwave-big-linux-ask-hot" and .verdict == "LINUX_HOT_SCAN_READY_NOT_LLM" and .field.state == "FOCUSED" and .field.top_facts[0].object == "bash" and .claim_boundary.json_used_in_hot_scan == false' <<<"$big_linux_ask_hot_json" >/dev/null
big_linux_boundary_hot_json="$("$llmwave_big" linux-ask-hot --hot-pack "$tmp_linux_hot" --query "package installed does not prove binary is running" --top-k 3 --format json)"
jq -e '.verdict == "LINUX_HOT_SCAN_READY_NOT_LLM" and .field.state == "BOUNDARY_FOCUSED" and .field.top_facts[0].polarity == "negative" and .claim_boundary.cache_only_execution_proven == false' <<<"$big_linux_boundary_hot_json" >/dev/null
big_linux_hot_eval_json="$("$llmwave_big" linux-hot-eval --hot-pack "$tmp_linux_hot" --top-k 3 --format json)"
jq -e '.mode == "llmwave-big-linux-hot-eval" and .verdict == "LINUX_HOT_EVAL_PASS_NOT_LLM" and .metrics.total == 4 and .metrics.passed == 4 and .metrics.lexical_duel_wins >= 1 and .claim_boundary.linux_domain_eval_ready == true and .claim_boundary.broad_chat_llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_linux_hot_eval_json" >/dev/null
big_linux_domain_run_json="$("$llmwave_big" linux-domain-run --hot-pack "$tmp_linux_hot" --query "which package provides command bash" --top-k 3 --format json)"
jq -e '.mode == "llmwave-big-linux-domain-run" and .verdict == "LINUX_DOMAIN_LLMWAVE_READY_NOT_GENERAL_LLM" and .query_wave.command_anchor == "bash" and .verifier.answer_allowed == true and .feedback_learning.feedback_packet_preview_ready == true' <<<"$big_linux_domain_run_json" >/dev/null
jq -e '.claim_boundary.linux_domain_core_ready == true and .claim_boundary.broad_chat_llm_ready == false and .claim_boundary.cache_only_execution_proven == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_linux_domain_run_json" >/dev/null
big_linux_cache_proof_json="$("$llmwave_big" linux-cache-proof --hot-pack "$tmp_linux_hot" --query "which package provides command bash" --iterations 2 --warmup-iterations 1 --samples 2 --format json)"
jq -e '.mode == "llmwave-big-linux-cache-proof" and .verdict == "LINUX_CACHE_ONLY_EXECUTION_PROVEN" and .runtime_contract.records_loaded_from == "laf-header-plus-fixed-record-section-only" and .runtime_contract.labels_read_from_packet == false and .runtime_contract.json_used_in_hot_loop == false and .runtime_contract.file_io_in_hot_loop == false and .runtime_contract.per_record_score_arrays == false and .claim_boundary.cache_only_execution_proven == true and .claim_boundary.broad_chat_llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_linux_cache_proof_json" >/dev/null
big_linux_pmu_cache_json="$("$llmwave_big" linux-pmu-cache-proof --hot-pack "$tmp_linux_hot" --query "which package provides command bash" --iterations 2 --warmup-iterations 1 --samples 2 --max-cache-miss-rate 1.0 --format json)"
jq -e '. as $r | $r.mode == "llmwave-big-linux-pmu-cache-proof" and (["LINUX_PMU_CACHE_RESIDENCY_PROVEN","LINUX_PMU_CACHE_RESIDENCY_REVIEW","LINUX_PMU_CACHE_RESIDENCY_BLOCKED"] | index($r.verdict)) and $r.software_runtime_contract.records_loaded_from == "laf-header-plus-fixed-record-section-only" and $r.claim_boundary.software_cache_only_execution_proven == true and (["MEASURED","NO_REFERENCES","BLOCKED"] | index($r.pmu.counter_status)) and $r.claim_boundary.broad_chat_llm_ready == false and $r.claim_boundary.nonlinear_memory_proven == false' <<<"$big_linux_pmu_cache_json" >/dev/null
tmp_linux_residual="$tmp_linux_active/linux-active.lrf"
big_linux_pack_residual_json="$("$llmwave_big" linux-pack-residual --atlas-dir "$tmp_linux_active" --max-active-facts 7 --out "$tmp_linux_residual" --format json)"
jq -e '.mode == "llmwave-big-linux-pack-residual" and .verdict == "LINUX_SCHEMA_RESIDUAL_PACKET_READY_NOT_PROOF" and .packet.schema_record_count == 1 and .packet.residual_record_count == 2 and .packet.fallback_record_count == 5 and .packet.represented_fact_count == 7 and .packet.binary_hot_sections_fit_6m == true and .economics.residual_saving_bytes > 0 and .claim_boundary.binary_schema_residual_memory_written == true and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_linux_pack_residual_json" >/dev/null
test -s "$tmp_linux_residual"
big_linux_residual_proof_json="$("$llmwave_big" linux-residual-proof --residual-pack "$tmp_linux_residual" --query "which package provides command bash" --top-k 3 --iterations 2 --warmup-iterations 1 --samples 2 --format json)"
jq -e '.mode == "llmwave-big-linux-residual-proof" and .verdict == "LINUX_SCHEMA_RESIDUAL_MEMORY_PROVEN" and .runtime_contract.records_loaded_from == "lrf-header-plus-schema-residual-fallback-sections" and .runtime_contract.schema_residual_binary_sections_scanned == true and .runtime_contract.json_used_in_hot_loop == false and .runtime_contract.labels_used_in_hot_loop == false and .runtime_contract.per_record_score_arrays == false and .residual_pack.beats_direct_fixed64 == true and .eval.metrics.total == 4 and .eval.metrics.passed == 4 and .semantic_atom_contract.verdict == "LINUX_ROLE_COMPLETE_SEMANTIC_ATOMS_PROVEN" and .semantic_atom_contract.gates.semantic_atom_contract_proven == true and .semantic_atom_contract.hot_record_policy.surface_text_length_is_not_mass == true and .semantic_atom_contract.metrics.role_complete_rate == 1 and .spectral_center.verdict == "LINUX_SPECTRAL_CENTER_PROVEN" and .spectral_center.gates.spectral_center_proven == true and .spectral_center.center_contract.version == "field-center-contract-v1" and .spectral_center.center_contract.center_kind == "memory" and .spectral_center.center_contract.read_only == true and .spectral_center.center_contract.decision_affects_authority == false and .spectral_center.center_contract.near_miss_rejected == true and .claim_boundary.semantic_atom_gate_required == true and .claim_boundary.semantic_atom_contract_proven == true and .claim_boundary.spectral_center_gate_required == true and .claim_boundary.spectral_center_proven == true and .claim_boundary.nonlinear_memory_proven == true and .claim_boundary.linux_profile_nonlinear_memory_proven == true and .claim_boundary.broad_chat_llm_ready == false and .claim_boundary.exposure_layer_ready == false' <<<"$big_linux_residual_proof_json" >/dev/null
tmp_linux_residual_v1="$tmp_linux_active/legacy-v1.lrf"
printf 'LLMWLRF1\001\000\000\000' >"$tmp_linux_residual_v1"
set +e
big_linux_residual_v1_json="$("$llmwave_big" linux-residual-proof --residual-pack "$tmp_linux_residual_v1" --query "which package provides command bash" --format json)"
big_linux_residual_v1_status=$?
set -e
test "$big_linux_residual_v1_status" -eq 3
jq -e '.mode == "llmwave-big-linux-residual-proof" and .verdict == "LRF_REPACK_REQUIRED" and .detected.magic == "LLMWLRF1" and .required.magic == "LLMWLRF2" and .safe_to_use == false and .claim_boundary.proof_not_run == true and .claim_boundary.nonlinear_memory_proven == false and (.repack_command | contains("linux-pack-residual"))' <<<"$big_linux_residual_v1_json" >/dev/null
big_linux_atlas_projection_json="$("$llmwave_big" linux-atlas-projection --atlas-dir "$tmp_linux_active" --hot-pack "$tmp_linux_hot" --residual-pack "$tmp_linux_residual" --query "which package provides command bash" --top-k 3 --iterations 2 --warmup-iterations 1 --samples 2 --format json)"
jq -e '.mode == "llmwave-big-linux-atlas-6mb-projection" and .verdict == "LINUX_ATLAS_6MB_COGNITIVE_PROJECTION_READY" and .projection.active_fact_count == 7 and .projection.lossless_atlas_storage_in_6m == true and .projection.useful_cognitive_projection_in_6m == true and .runtime.laf_cache_only_execution_proven == true and .runtime.lrf_nonlinear_memory_proven == true and .gates.final_gate_ready == true and .claim_boundary.useful_atlas_projection_ready == true and .claim_boundary.atlas_lossless_storage_in_6m == false and .claim_boundary.global_nonlinear_memory_proven == false and .claim_boundary.general_llm_ready == false' <<<"$big_linux_atlas_projection_json" >/dev/null
tmp_linux_exposure="$(mktemp -d)"
mkdir -p "$tmp_linux_exposure/facts"
tmp_linux_exposure_facts="$tmp_linux_exposure/facts/base.jsonl"
cat >"$tmp_linux_exposure_facts" <<JSONL
{"fact_id":"linux.exposure.package.sshd","layer":"linux-knowledge","domain":"package-file-list","route":"linux.package.binary","subject":"openssh-server","relation":"provides binary","object":"/usr/sbin/sshd","polarity":"positive","confidence":90,"evidence":{"source_kind":"fixture","path":"fixture","line":1,"extractor":"fixture"}}
{"fact_id":"linux.exposure.package.systemctl","layer":"linux-knowledge","domain":"package-file-list","route":"linux.package.binary","subject":"systemd","relation":"provides binary","object":"/usr/bin/systemctl","polarity":"positive","confidence":90,"evidence":{"source_kind":"fixture","path":"fixture","line":2,"extractor":"fixture"}}
{"fact_id":"linux.exposure.socket.any","layer":"runtime-snapshot","domain":"runtime-socket","route":"linux.socket.runtime","subject":"tcp","relation":"listens on","object":"00000000:0016","polarity":"positive","confidence":90,"evidence":{"source_kind":"fixture","path":"fixture","line":3,"extractor":"fixture"}}
{"fact_id":"linux.exposure.socket.local","layer":"runtime-snapshot","domain":"runtime-socket","route":"linux.socket.runtime","subject":"tcp","relation":"listens on","object":"0100007F:1F90","polarity":"positive","confidence":90,"evidence":{"source_kind":"fixture","path":"fixture","line":4,"extractor":"fixture"}}
{"fact_id":"linux.exposure.systemd.ssh","layer":"linux-knowledge","domain":"systemd","route":"linux.systemd.exec","subject":"ssh.service","relation":"execstart","object":"/usr/sbin/sshd","polarity":"positive","confidence":88,"evidence":{"source_kind":"fixture","path":"fixture","line":5,"extractor":"fixture"}}
{"fact_id":"linux.exposure.firewall.allow","layer":"runtime-snapshot","domain":"runtime-firewall","route":"linux.firewall.runtime","subject":"ufw","relation":"allows port","object":"22/tcp","polarity":"positive","confidence":92,"evidence":{"source_kind":"fixture","path":"fixture","line":6,"extractor":"fixture"}}
{"fact_id":"linux.exposure.boundary.socket","layer":"negative-boundary","domain":"linux-boundary","route":"linux.boundary.socket","subject":"port listening","relation":"does not prove","object":"firewall allows external packets","polarity":"negative","confidence":95,"evidence":{"source_kind":"fixture","path":"fixture","line":7,"extractor":"fixture"}}
{"fact_id":"linux.exposure.boundary.package","layer":"negative-boundary","domain":"linux-boundary","route":"linux.boundary.package","subject":"package installed","relation":"does not prove","object":"binary is running","polarity":"negative","confidence":95,"evidence":{"source_kind":"fixture","path":"fixture","line":8,"extractor":"fixture"}}
JSONL
tmp_linux_exposure_residual="$tmp_linux_exposure/linux-exposure.lrf"
"$llmwave_big" linux-pack-residual --atlas-dir "$tmp_linux_exposure" --max-active-facts 8 --out "$tmp_linux_exposure_residual" --format json >/dev/null
big_linux_exposure_json="$("$llmwave_big" linux-exposure-run --residual-pack "$tmp_linux_exposure_residual" --max-candidates 8 --format json)"
jq -e '.mode == "llmwave-big-linux-exposure-run" and .verdict == "LINUX_EXPOSURE_REASONING_READY_NOT_SCANNER" and .exposure_field.state == "EXPOSURE_CONFIRMED_REVIEW" and .exposure_field.safe_to_claim_external_exposure == true and .exposure_field.firewall_allow_fact_count == 1 and .eval.metrics.total == 4 and .eval.metrics.passed == 4 and .claim_boundary.exposure_layer_ready == true and .claim_boundary.vulnerability_scan_ready == false and .claim_boundary.broad_chat_llm_ready == false' <<<"$big_linux_exposure_json" >/dev/null
jq -e '.exposure_field.candidates[] | select(.endpoint == "0100007F:1F90" and .verdict == "LOCAL_ONLY_NOT_EXTERNAL_EXPOSURE")' <<<"$big_linux_exposure_json" >/dev/null
tmp_linux_chat="$(mktemp -d)"
mkdir -p "$tmp_linux_chat/facts"
tmp_linux_chat_facts="$tmp_linux_chat/facts/base.jsonl"
cat >"$tmp_linux_chat_facts" <<JSONL
{"fact_id":"linux.chat.bash","layer":"linux-knowledge","domain":"apt-command-index","route":"linux.apt.command.provider","subject":"bash","relation":"provided by package","object":"bash","polarity":"positive","confidence":90,"evidence":{"source_kind":"fixture","path":"fixture","line":1,"extractor":"fixture"}}
{"fact_id":"linux.chat.systemctl","layer":"linux-knowledge","domain":"package-file-list","route":"linux.package.binary","subject":"systemd","relation":"provides binary","object":"/usr/bin/systemctl","polarity":"positive","confidence":90,"evidence":{"source_kind":"fixture","path":"fixture","line":2,"extractor":"fixture"}}
{"fact_id":"linux.chat.socket.any","layer":"runtime-snapshot","domain":"runtime-socket","route":"linux.socket.runtime","subject":"tcp","relation":"listens on","object":"00000000:0016","polarity":"positive","confidence":90,"evidence":{"source_kind":"fixture","path":"fixture","line":3,"extractor":"fixture"}}
{"fact_id":"linux.chat.socket.local","layer":"runtime-snapshot","domain":"runtime-socket","route":"linux.socket.runtime","subject":"tcp","relation":"listens on","object":"0100007F:1F90","polarity":"positive","confidence":90,"evidence":{"source_kind":"fixture","path":"fixture","line":4,"extractor":"fixture"}}
{"fact_id":"linux.chat.systemd.ssh","layer":"linux-knowledge","domain":"systemd","route":"linux.systemd.exec","subject":"ssh.service","relation":"execstart","object":"/usr/sbin/sshd","polarity":"positive","confidence":88,"evidence":{"source_kind":"fixture","path":"fixture","line":5,"extractor":"fixture"}}
{"fact_id":"linux.chat.boundary.socket","layer":"negative-boundary","domain":"linux-boundary","route":"linux.boundary.socket","subject":"port listening","relation":"does not prove","object":"firewall allows external packets","polarity":"negative","confidence":95,"evidence":{"source_kind":"fixture","path":"fixture","line":6,"extractor":"fixture"}}
{"fact_id":"linux.chat.boundary.package","layer":"negative-boundary","domain":"linux-boundary","route":"linux.boundary.package","subject":"package installed","relation":"does not prove","object":"binary is running","polarity":"negative","confidence":95,"evidence":{"source_kind":"fixture","path":"fixture","line":7,"extractor":"fixture"}}
{"fact_id":"linux.chat.boundary.cve","layer":"negative-boundary","domain":"linux-boundary","route":"linux.boundary.cve","subject":"vulnerable package","relation":"does not prove","object":"runtime exposure","polarity":"negative","confidence":95,"evidence":{"source_kind":"fixture","path":"fixture","line":8,"extractor":"fixture"}}
JSONL
tmp_linux_chat_residual="$tmp_linux_chat/linux-chat.lrf"
"$llmwave_big" linux-pack-residual --atlas-dir "$tmp_linux_chat" --max-active-facts 8 --out "$tmp_linux_chat_residual" --format json >/dev/null
big_linux_chat_json="$("$llmwave_big" linux-chat-run --residual-pack "$tmp_linux_chat_residual" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-chat-run" and .verdict == "LINUX_PROFILE_BROAD_CHAT_READY_NOT_GENERAL_LLM" and .eval.metrics.total == 5 and .eval.metrics.passed == 5 and .claim_boundary.broad_chat_llm_ready == true and .claim_boundary.linux_profile_broad_chat_ready == true and .claim_boundary.general_llm_ready == false and .claim_boundary.vulnerability_scan_ready == false' <<<"$big_linux_chat_json" >/dev/null
jq -e '.turns[] | select(.intent == "external_exposure" and (.answer | ascii_downcase | contains("not confirmed")))' <<<"$big_linux_chat_json" >/dev/null
big_linux_chat_v1_eval_json="$("$llmwave_big" linux-chat-v1-eval --residual-pack "$tmp_linux_chat_residual" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-chat-v1" and .verdict == "LINUX_CHAT_V1_READY_NOT_GENERAL_LLM" and .eval.total == 6 and .eval.passed == 6 and .claim_boundary.linux_chat_v1_ready == true and .claim_boundary.general_llm_ready == false and .claim_boundary.broad_chat_llm_ready == false' <<<"$big_linux_chat_v1_eval_json" >/dev/null
jq -e '.turns[] | select(.context_resolution.correction_applied == true and .intent == "command_provider" and (.answer | ascii_downcase | contains("systemctl")))' <<<"$big_linux_chat_v1_eval_json" >/dev/null
jq -e '.turns[] | select(.context_resolution.used_previous_topic == true and .intent == "external_exposure" and (.answer | ascii_downcase | contains("not confirmed")) and (.rejected_shortcuts | index("listener_implies_external_exposure")))' <<<"$big_linux_chat_v1_eval_json" >/dev/null
tmp_linux_chat_v1_script="$tmp_linux_chat/linux-chat-v1.script"
cat >"$tmp_linux_chat_v1_script" <<SCRIPT
Which package provides command bash?
I meant systemctl.
What listeners are present?
Does that mean external exposure?
SCRIPT
big_linux_chat_v1_script_json="$("$llmwave_big" linux-chat-v1 --residual-pack "$tmp_linux_chat_residual" --script "$tmp_linux_chat_v1_script" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-chat-v1" and .turn_count == 4 and .feedback_memory.context_rewrites == 2 and .feedback_memory.corrections_applied == 1 and .claim_boundary.general_llm_ready == false' <<<"$big_linux_chat_v1_script_json" >/dev/null
tmp_linux_chat_v2_memory="$tmp_linux_chat/linux-chat-v2.lwm"
big_linux_chat_v2_eval_json="$("$llmwave_big" linux-chat-v2-eval --residual-pack "$tmp_linux_chat_residual" --memory "$tmp_linux_chat_v2_memory" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-chat-v2" and .verdict == "LINUX_CHAT_V2_PERSISTENT_WAVE_LEARNING_READY_NOT_GENERAL_LLM" and .eval.total == 6 and .eval.passed == 6 and .eval.deltas_written == 2 and .eval.memory_lift_observed == true and .eval.answer_changed_due_to_wave_memory == true and .eval.negative_lane_replay_observed == true and .eval.unrelated_route_preserved == true and .claim_boundary.dialogue_learning_ready == true and .claim_boundary.persistent_wave_memory_ready == true and .claim_boundary.session_log_used_as_memory == false and .claim_boundary.general_llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_linux_chat_v2_eval_json" >/dev/null
jq -e '.turns[] | select(.verifier_state == "GROUNDED_BY_WAVE_MEMORY" and .answer_changed_due_to_wave_memory == true and (.answer | ascii_downcase | contains("foopkg")))' <<<"$big_linux_chat_v2_eval_json" >/dev/null
jq -e '.turns[] | select(.verifier_state == "BOUNDARY_WITH_LEARNED_ANTI_WAVE" and .memory_effect.learned_negative_lanes_active == true)' <<<"$big_linux_chat_v2_eval_json" >/dev/null
test -s "$tmp_linux_chat_v2_memory"
jq -e '.record_bytes == 32 and .record_count == 2 and (.records[] | select(.delta_state == "POSITIVE_DELTA")) and (.records[] | select(.delta_state == "NEGATIVE_DELTA"))' "$tmp_linux_chat_v2_memory" >/dev/null
tmp_linux_vpn_memory="$tmp_linux_chat/linux-vpn.lwm"
big_linux_vpn_train_eval_json="$("$llmwave_big" linux-vpn-train-eval --residual-pack "$tmp_linux_chat_residual" --memory "$tmp_linux_vpn_memory" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-vpn-train-eval" and .verdict == "LINUX_VPN_LOCAL_TRAINING_READY_NOT_AUTOCONFIG" and .training.verdict == "LINUX_VPN_WAVE_MEMORY_TRAINED_NOT_SYSTEM_MUTATION" and .training.inserted_delta_count == 6 and .training.memory.record_count == 6 and .eval.total == 5 and .eval.passed == 5 and .claim_boundary.local_vpn_training_ready == true and .claim_boundary.local_system_mutation_done == false and .claim_boundary.secrets_read == false and .claim_boundary.secrets_printed == false and .claim_boundary.general_llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_linux_vpn_train_eval_json" >/dev/null
jq -e '.chat.turns[] | select(.intent == "vpn_wireguard_setup" and .verifier_state == "GROUNDED_BY_WAVE_MEMORY" and (.answer | ascii_downcase | contains("wg-quick up")))' <<<"$big_linux_vpn_train_eval_json" >/dev/null
jq -e '.chat.turns[] | select(.intent == "vpn_secret_boundary" and .verifier_state == "BOUNDARY_WITH_LEARNED_ANTI_WAVE" and (.answer | ascii_downcase | contains("do not print private keys")))' <<<"$big_linux_vpn_train_eval_json" >/dev/null
test -s "$tmp_linux_vpn_memory"
big_linux_vpn_action_plan_json="$("$llmwave_big" linux-vpn-action-plan --text "turn off wireguard vpn wg0" --format json)"
jq -e '.mode == "llmwave-big-linux-vpn-action-plan" and .verdict == "LINUX_VPN_ACTION_PLAN_READY_NOT_EXECUTED" and .action == "down" and .backend == "wireguard" and .target == "wg0" and .plan.argv == ["sudo","wg-quick","down","wg0"] and .claim_boundary.local_system_mutation_done == false and .claim_boundary.secrets_read == false and .claim_boundary.secrets_printed == false' <<<"$big_linux_vpn_action_plan_json" >/dev/null
big_linux_vpn_control_json="$("$llmwave_big" linux-vpn-control --action down --backend wireguard --target wg0 --format json)"
jq -e '.mode == "llmwave-big-linux-vpn-control" and .verdict == "LINUX_VPN_CONTROL_DRY_RUN_READY" and .plan.argv == ["sudo","wg-quick","down","wg0"] and .execution.executed == false and .claim_boundary.local_system_mutation_done == false' <<<"$big_linux_vpn_control_json" >/dev/null
big_linux_vpn_blocked_json="$("$llmwave_big" linux-vpn-control --action down --backend wireguard --target wg0 --execute --format json)"
jq -e '.mode == "llmwave-big-linux-vpn-control" and .verdict == "LINUX_VPN_CONTROL_BLOCKED_CONFIRMATION_REQUIRED" and .execution.requested == true and .execution.blocked == true and .execution.executed == false and .claim_boundary.local_system_mutation_done == false' <<<"$big_linux_vpn_blocked_json" >/dev/null
big_linux_query_wave_json="$("$llmwave_big" linux-query-wave --text "Is ssh externally exposed?" --format json)"
jq -e '.mode == "llmwave-big-linux-query-wave" and .verdict == "LINUX_QUERY_WAVE_READY_NOT_ANSWER" and .query_wave.intent == "external_exposure" and (.query_wave.forbidden_shortcuts | index("listener_implies_external_exposure")) and .claim_boundary.general_llm_ready == false' <<<"$big_linux_query_wave_json" >/dev/null
big_linux_reason_json="$("$llmwave_big" linux-reason-run --residual-pack "$tmp_linux_chat_residual" --text "Is this machine externally exposed?" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-reason-run" and .verdict == "LINUX_PROFILE_REASONING_READY_NOT_GENERAL_LLM" and .decision.state == "EXPOSURE_NOT_CONFIRMED" and (.decision.answer | ascii_downcase | contains("not confirmed")) and .claim_boundary.linux_profile_reasoning_ready == true and .claim_boundary.general_llm_ready == false' <<<"$big_linux_reason_json" >/dev/null
jq -e '.anti_wave_hits[] | select(.shortcut == "listener_implies_external_exposure")' <<<"$big_linux_reason_json" >/dev/null
tmp_linux_runtime_snapshot="$tmp_linux_chat/runtime-snapshot.json"
cat >"$tmp_linux_runtime_snapshot" <<JSON
{
  "firewall": {
    "engine": "ufw",
    "rules": [
      {"action": "allow", "port": 22, "protocol": "tcp", "scope": "external"}
    ]
  }
}
JSON
big_linux_snapshot_import_json="$("$llmwave_big" linux-snapshot-import --snapshot "$tmp_linux_runtime_snapshot" --format json)"
jq -e '.mode == "llmwave-big-linux-snapshot-import" and .verdict == "LINUX_RUNTIME_SNAPSHOT_IMPORTED_NOT_SCANNER" and .overlay.fact_count == 1 and .overlay.firewall_allow_fact_count == 1 and .overlay.commands_executed == false and .claim_boundary.side_effect_free == true and .claim_boundary.confirms_exposure_by_itself == false' <<<"$big_linux_snapshot_import_json" >/dev/null
big_linux_exposure_snapshot_json="$("$llmwave_big" linux-exposure-run --residual-pack "$tmp_linux_chat_residual" --runtime-snapshot "$tmp_linux_runtime_snapshot" --max-candidates 8 --format json)"
jq -e '.mode == "llmwave-big-linux-exposure-run" and .exposure_field.state == "EXPOSURE_CONFIRMED_REVIEW" and .runtime_snapshot_overlay.firewall_allow_fact_count == 1 and .claim_boundary.external_exposure_confirmed == true and .claim_boundary.vulnerability_scan_ready == false' <<<"$big_linux_exposure_snapshot_json" >/dev/null
big_linux_reason_snapshot_json="$("$llmwave_big" linux-reason-run --residual-pack "$tmp_linux_chat_residual" --runtime-snapshot "$tmp_linux_runtime_snapshot" --text "Is this machine externally exposed?" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-reason-run" and .decision.state == "EXPOSURE_CONFIRMED_REVIEW" and .runtime_snapshot_overlay.firewall_allow_fact_count == 1 and (.evidence_chain[] | select(.route == "linux.firewall.runtime")) and .claim_boundary.network_scanner_ready == false' <<<"$big_linux_reason_snapshot_json" >/dev/null
tmp_linux_profile_suite="$tmp_linux_chat/linux-profile-suite.json"
big_linux_profile_suite_json="$("$llmwave_big" linux-broad-suite-build --residual-pack "$tmp_linux_chat_residual" --cases 32 --out "$tmp_linux_profile_suite" --format json)"
jq -e '.mode == "llmwave-big-linux-broad-suite-build" and .verdict == "LINUX_BROAD_SUITE_READY_NOT_EVAL" and .suite.case_count == 32 and .claim_boundary.general_llm_ready == false' <<<"$big_linux_profile_suite_json" >/dev/null
test -s "$tmp_linux_profile_suite"
tmp_linux_profile_eval="$tmp_linux_chat/linux-profile-eval.json"
big_linux_profile_eval_json="$("$llmwave_big" linux-broad-eval-run --residual-pack "$tmp_linux_chat_residual" --suite "$tmp_linux_profile_suite" --out "$tmp_linux_profile_eval" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-broad-eval-run" and .verdict == "LINUX_PROFILE_BROAD_EVAL_PASS_NOT_GENERAL_LLM" and .metrics.total == 32 and .metrics.passed == 32 and .metrics.shortcut_rejection_rate == 1 and .metrics.exposure_overclaim_rate == 0 and .claim_boundary.linux_profile_broad_eval_ready == true and .claim_boundary.general_llm_ready == false' <<<"$big_linux_profile_eval_json" >/dev/null
test -s "$tmp_linux_profile_eval"
big_linux_profile_claim_json="$("$llmwave_big" linux-profile-claim-gate --residual-pack "$tmp_linux_chat_residual" --broad-eval "$tmp_linux_profile_eval" --format json)"
jq -e '.mode == "llmwave-big-linux-profile-claim-gate" and .verdict == "LINUX_PROFILE_REASONING_READY_NOT_GENERAL_LLM" and .requirements.linux_profile_nonlinear_memory_proven == true and .requirements.broad_eval_pass_rate_ok == true and .claim_boundary.linux_profile_reasoning_ready == true and .claim_boundary.linux_profile_broad_chat_ready == true and .claim_boundary.general_llm_ready == false and .claim_boundary.vulnerability_scanner_ready == false' <<<"$big_linux_profile_claim_json" >/dev/null
tmp_linux_heldout_suite="$tmp_linux_chat/linux-heldout-suite.json"
big_linux_heldout_suite_json="$("$llmwave_big" linux-heldout-suite-build --residual-pack "$tmp_linux_chat_residual" --cases 32 --out "$tmp_linux_heldout_suite" --format json)"
jq -e '.mode == "llmwave-big-linux-heldout-suite-build" and .verdict == "LINUX_HELDOUT_SUITE_READY_NOT_EVAL" and .suite.case_count == 32 and .controls.near_collision_cases > 0 and .controls.shortcut_control_cases > 0 and .claim_boundary.general_llm_ready == false' <<<"$big_linux_heldout_suite_json" >/dev/null
test -s "$tmp_linux_heldout_suite"
tmp_linux_heldout_eval="$tmp_linux_chat/linux-heldout-eval.json"
big_linux_heldout_eval_json="$("$llmwave_big" linux-heldout-eval-run --residual-pack "$tmp_linux_chat_residual" --suite "$tmp_linux_heldout_suite" --out "$tmp_linux_heldout_eval" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-heldout-eval-run" and .verdict == "LINUX_PROFILE_HELDOUT_EVAL_PASS_NOT_GENERAL_LLM" and .metrics.total == 32 and .metrics.passed == 32 and .metrics.false_positive_rate == 0 and .claim_boundary.general_llm_ready == false' <<<"$big_linux_heldout_eval_json" >/dev/null
test -s "$tmp_linux_heldout_eval"
tmp_linux_center_learning_memory="$tmp_linux_chat/linux-center-learning.lwm"
big_linux_center_learn_json="$("$llmwave_big" linux-center-learn --residual-pack "$tmp_linux_chat_residual" --memory "$tmp_linux_center_learning_memory" --heldout-eval "$tmp_linux_heldout_eval" --max-facts 4 --reset-memory --format json)"
jq -e '.mode == "llmwave-big-linux-center-learn" and .verdict == "LINUX_DYNAMIC_CENTER_LEARNING_READY_NOT_GENERAL_LLM" and .dynamic_center_learning.enabled == true and .dynamic_center_learning.operations.confirmed_center_reinforcement == true and .dynamic_center_learning.operations.correction_residual_write == true and .dynamic_center_learning.operations.reject_anti_center_write == true and .dynamic_center_learning.operations.residual_cluster_promotion == true and .dynamic_center_learning.operations.center_split_available == true and .dynamic_center_learning.operations.weak_candidate_decay == true and .dynamic_center_learning.operations.verified_center_protection == true and .dynamic_center_learning.operations.candidate_quarantine_write == true and .dynamic_center_learning.operations.candidate_center_admission == true and .dynamic_center_learning.operations.evidence_weighted_update == true and .dynamic_center_learning.operations.drift_budget_enforced == true and .dynamic_center_learning.safety.bad_feedback_quarantined == true and .dynamic_center_learning.safety.verified_center_drift_blocked == true and .dynamic_center_learning.before_after.target_query_improved == true and .dynamic_center_learning.before_after.memory_lift_observed == true and .dynamic_center_learning.before_after.anti_center_replay_observed == true and .dynamic_center_learning.before_after.false_positive_rate_regressed == false and .dynamic_center_learning.before_after.heldout_regressed == false and .dynamic_center_learning.before_after.unrelated_route_preserved == true and .claim_boundary.linux_profile_dynamic_learning_ready == true and .claim_boundary.general_llm_ready == false and .claim_boundary.global_nonlinear_memory_proven == false and .claim_boundary.proof_grade_profile_scale == false' <<<"$big_linux_center_learn_json" >/dev/null
test -s "$tmp_linux_center_learning_memory"
jq -e '.center_record_count > 0 and .residual_record_count >= 2 and .anti_center_record_count >= 2 and .candidate_center_record_count >= 2 and .quarantine_record_count >= 3 and (.candidate_centers[] | select(.admitted == true and .evidence_weight >= 120)) and (.quarantine[] | select(.rejected == true and .reason == "verified_center_drift_budget")) and (.wave_deltas | length) >= 5' "$tmp_linux_center_learning_memory" >/dev/null
tmp_linux_chat_profile_memory="$tmp_linux_chat/linux-chat-profile.lwm"
tmp_linux_chat_profile_center_memory="$tmp_linux_chat/linux-chat-profile-center.lwm"
tmp_linux_chat_profile_vpn_memory="$tmp_linux_chat/linux-chat-profile-vpn.lwm"
big_linux_chat_profile_gate_json="$("$llmwave_big" linux-chat-profile-gate --residual-pack "$tmp_linux_chat_residual" --broad-eval "$tmp_linux_profile_eval" --heldout-eval "$tmp_linux_heldout_eval" --run-chat-learning-eval --chat-learning-memory "$tmp_linux_chat_profile_memory" --run-center-learning-eval --center-learning-memory "$tmp_linux_chat_profile_center_memory" --run-vpn-training-eval --vpn-memory "$tmp_linux_chat_profile_vpn_memory" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-profile-claim-gate" and .verdict == "LINUX_PROFILE_REASONING_READY_NOT_GENERAL_LLM" and .chat_target.target == "LLMWAVE_LINUX_CHAT_PROFILE_V1" and .chat_target.ready == false and (.chat_target.blocked_by | index("proof_grade_linux_profile_nonlinear_memory")) and .chat_target.global_llm_ready == false and .chat_target.global_nonlinear_memory_proven == false and .memory_proof.proof_grade == false and .memory_proof.proof_grade_min_represented_facts == 65536 and .memory_proof.proof_grade_fact_count_ok == false and .requirements.linux_profile_nonlinear_memory_proven == true and .requirements.proof_grade_linux_profile_nonlinear_memory_proven == false and .requirements.proof_grade_fact_count_ok == false and .requirements.heldout_eval_present == true and .requirements.chat_learning_eval_present == true and .requirements.memory_lift_observed == true and .requirements.learned_anti_wave_observed == true and .requirements.center_learning_eval_present == true and .requirements.dynamic_center_learning_ready == true and .center_learning.candidate_quarantine_write == true and .center_learning.candidate_center_admission == true and .center_learning.drift_budget_enforced == true and .center_learning.bad_feedback_quarantined == true and .center_learning.verified_center_drift_blocked == true and .requirements.center_memory_lift_observed == true and .requirements.center_anti_replay_observed == true and .requirements.center_false_positive_regression_free == true and .requirements.center_heldout_regression_free == true and .requirements.center_unrelated_route_preserved == true and .requirements.vpn_training_ready == true and .requirements.vpn_secret_boundary_ready == true and .claim_boundary.general_llm_ready == false' <<<"$big_linux_chat_profile_gate_json" >/dev/null
test -s "$tmp_linux_chat_profile_memory"
test -s "$tmp_linux_chat_profile_center_memory"
test -s "$tmp_linux_chat_profile_vpn_memory"
tmp_linux_chat_core_cache="$tmp_linux_chat/cache"
tmp_linux_chat_core_missing_cache="$tmp_linux_chat/missing-cache"
big_linux_chat_core_missing_gate_json="$("$llmwave_big" linux-chat-core-gate --residual-pack "$tmp_linux_chat_residual" --dialogue-overlay "$tmp_linux_chat_profile_memory" --centers-overlay "$tmp_linux_chat_profile_center_memory" --vpn-overlay "$tmp_linux_chat_profile_vpn_memory" --broad-eval "$tmp_linux_profile_eval" --heldout-eval "$tmp_linux_heldout_eval" --cache-dir "$tmp_linux_chat_core_missing_cache" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-chat-core-gate" and .verdict == "LINUX_CHAT_CORE_CACHE_STALE" and .cache_status.cache_fresh == false and (.cache_status.stale_reasons | index("cache_manifest_missing")) and .profile_gate == null and .chat_core.safe_to_use_cache == false and .chat_core.safe_to_answer_from_cache == false and .token_economics.cache_total_bytes == 0 and .token_economics.cache_is_runtime_index_not_prompt_payload == true' <<<"$big_linux_chat_core_missing_gate_json" >/dev/null
test ! -e "$tmp_linux_chat_core_missing_cache/chat-core.manifest.json"
big_linux_chat_core_build_json="$("$llmwave_big" linux-chat-core-build --residual-pack "$tmp_linux_chat_residual" --dialogue-overlay "$tmp_linux_chat_profile_memory" --centers-overlay "$tmp_linux_chat_profile_center_memory" --vpn-overlay "$tmp_linux_chat_profile_vpn_memory" --broad-eval "$tmp_linux_profile_eval" --heldout-eval "$tmp_linux_heldout_eval" --cache-dir "$tmp_linux_chat_core_cache" --format json)"
jq -e '.mode == "llmwave-big-linux-chat-core-build" and .verdict == "LINUX_CHAT_CORE_CACHE_READY_NOT_GENERAL_LLM" and .manifest.profile_id == "linux-chat-core" and .manifest.cache_is_source_of_truth == false and .source_status.source_memory_loaded == true and .source_status.overlays_present == 3 and .token_economics.source_artifacts_estimated_tokens > 0 and .token_economics.cache_estimated_tokens > 0 and .token_economics.cache_is_runtime_index_not_prompt_payload == true and .claim_boundary.cache_is_source_of_truth == false and .claim_boundary.cache_is_runtime_index_not_prompt_payload == true and .claim_boundary.general_llm_ready == false' <<<"$big_linux_chat_core_build_json" >/dev/null
test -s "$tmp_linux_chat_core_cache/chat-core.hot"
test -s "$tmp_linux_chat_core_cache/chat-core.index.json"
test -s "$tmp_linux_chat_core_cache/chat-core.manifest.json"
jq -e '.represented_fact_count == 8 and (.route_index | length) > 0 and (.readout_facts | length) == 8 and .cache_contract.hot_cache_has_no_authority_without_gate == true' "$tmp_linux_chat_core_cache/chat-core.index.json" >/dev/null
big_linux_chat_core_gate_json="$("$llmwave_big" linux-chat-core-gate --residual-pack "$tmp_linux_chat_residual" --dialogue-overlay "$tmp_linux_chat_profile_memory" --centers-overlay "$tmp_linux_chat_profile_center_memory" --vpn-overlay "$tmp_linux_chat_profile_vpn_memory" --broad-eval "$tmp_linux_profile_eval" --heldout-eval "$tmp_linux_heldout_eval" --cache-dir "$tmp_linux_chat_core_cache" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-chat-core-gate" and .cache_status.cache_fresh == true and .cache_status.cache_is_source_of_truth == false and .chat_core.safe_to_use_cache == true and .chat_core.source_hash_matched == true and .chat_core.cache_is_source_of_truth == false and .chat_core.compatibility_wrapper_for_linux_chat_profile_gate == true and .claim_boundary.stale_cache_blocks_answer_authority == true and .claim_boundary.cache_is_runtime_index_not_prompt_payload == true and .claim_boundary.general_llm_ready == false' <<<"$big_linux_chat_core_gate_json" >/dev/null
big_linux_chat_core_ask_json="$("$llmwave_big" linux-chat-core-ask --text "which package provides command bash" --residual-pack "$tmp_linux_chat_residual" --dialogue-overlay "$tmp_linux_chat_profile_memory" --centers-overlay "$tmp_linux_chat_profile_center_memory" --vpn-overlay "$tmp_linux_chat_profile_vpn_memory" --cache-dir "$tmp_linux_chat_core_cache" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-chat-core-ask" and .verdict == "LINUX_CHAT_CORE_PACKET_READY_NOT_GENERAL_LLM" and .cache_status.cache_fresh == true and .grounded_packet.cache_fresh == true and .grounded_packet.answer_allowed == true and .grounded_packet.readout_source == "compiled_chat_core_index" and .grounded_packet.cache_is_runtime_index_not_prompt_payload == true and .grounded_packet.decision_state == "ANSWER_GROUNDED" and (.grounded_packet.answer | ascii_downcase | contains("bash")) and (.grounded_packet.evidence[] | select((.route == "linux.apt.command.provider" or .route == "linux.apt.command.package-command") and .role == "provider" and .subject == "bash" and .object == "bash" and (.memory_kind | length > 0))) and (.grounded_packet.compact_evidence[] | contains("subject=bash")) and .token_economics.grounded_packet_estimated_tokens > 0 and .token_economics.actual_answer_context_estimated_tokens == .token_economics.grounded_packet_estimated_tokens and .token_economics.estimated_tokens_saved_vs_source > 0 and .token_economics.estimated_tokens_saved_vs_cache_index > 0 and .token_economics.cache_is_runtime_index_not_prompt_payload == true and .claim_boundary.cache_is_source_of_truth == false and .claim_boundary.cache_is_runtime_index_not_prompt_payload == true' <<<"$big_linux_chat_core_ask_json" >/dev/null
big_linux_chat_core_ask_systemctl_json="$("$llmwave_big" linux-chat-core-ask --text "which package provides command systemctl" --residual-pack "$tmp_linux_chat_residual" --dialogue-overlay "$tmp_linux_chat_profile_memory" --centers-overlay "$tmp_linux_chat_profile_center_memory" --vpn-overlay "$tmp_linux_chat_profile_vpn_memory" --cache-dir "$tmp_linux_chat_core_cache" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-chat-core-ask" and .verdict == "LINUX_CHAT_CORE_PACKET_READY_NOT_GENERAL_LLM" and .grounded_packet.answer_allowed == true and .grounded_packet.decision_state == "ANSWER_GROUNDED" and (.grounded_packet.answer | contains("systemd")) and (.grounded_packet.evidence[] | select(.route == "linux.package.binary" and .role == "provider" and .subject == "systemd" and .relation == "provides binary" and .object == "/usr/bin/systemctl" and (.memory_kind | length > 0)))' <<<"$big_linux_chat_core_ask_systemctl_json" >/dev/null
printf '\n# stale check\n' >>"$tmp_linux_chat_profile_center_memory"
big_linux_chat_core_stale_json="$("$llmwave_big" linux-chat-core-ask --text "which package provides command bash" --residual-pack "$tmp_linux_chat_residual" --dialogue-overlay "$tmp_linux_chat_profile_memory" --centers-overlay "$tmp_linux_chat_profile_center_memory" --vpn-overlay "$tmp_linux_chat_profile_vpn_memory" --cache-dir "$tmp_linux_chat_core_cache" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-chat-core-ask" and .verdict == "LINUX_CHAT_CORE_CACHE_STALE" and .cache_status.cache_fresh == false and (.cache_status.stale_reasons | index("source_memory_hash_changed")) and .grounded_packet.answer_allowed == false and .grounded_packet.decision_state == "CACHE_STALE_NO_AUTHORITY"' <<<"$big_linux_chat_core_stale_json" >/dev/null
tmp_linux_feedback="$tmp_linux_chat/linux-feedback.json"
big_linux_feedback_build_json="$("$llmwave_big" linux-feedback-build --residual-pack "$tmp_linux_chat_residual" --text "Is this machine externally exposed?" --decision reject --note "profile shortcut rejection" --out "$tmp_linux_feedback" --format json)"
jq -e '.mode == "llmwave-big-linux-feedback-build" and .verdict == "LINUX_FEEDBACK_PACKET_READY_NOT_TRAINING" and (.packet.negative_lanes | length) > 0 and .claim_boundary.general_llm_ready == false' <<<"$big_linux_feedback_build_json" >/dev/null
test -s "$tmp_linux_feedback"
big_linux_feedback_apply_json="$("$llmwave_big" linux-feedback-apply --residual-pack "$tmp_linux_chat_residual" --feedback "$tmp_linux_feedback" --text "Is this machine externally exposed?" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-feedback-apply" and .verdict == "LINUX_FEEDBACK_MEMORY_APPLIED_NOT_GENERAL_TRAINING" and .applied.negative_lanes_matched > 0 and .after.learned_negative_lanes_active == true and .claim_boundary.general_llm_ready == false' <<<"$big_linux_feedback_apply_json" >/dev/null
big_linux_decision_search_json="$("$llmwave_big" linux-decision-search --residual-pack "$tmp_linux_chat_residual" --text "Is this machine externally exposed?" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-decision-search" and .verdict == "LINUX_DECISION_SEARCH_READY_NOT_SCANNER" and (.decision_search.safe_next_checks | length) > 0 and (.decision_search.missing_evidence | index("matching firewall allow for same endpoint/port")) and .claim_boundary.network_scanner_ready == false' <<<"$big_linux_decision_search_json" >/dev/null
big_linux_decision_search_snapshot_json="$("$llmwave_big" linux-decision-search --residual-pack "$tmp_linux_chat_residual" --runtime-snapshot "$tmp_linux_runtime_snapshot" --text "Is this machine externally exposed?" --max-facts 4 --format json)"
jq -e '.mode == "llmwave-big-linux-decision-search" and .verdict == "LINUX_DECISION_SEARCH_READY_NOT_SCANNER" and .decision_search.state == "ANSWER_ALREADY_GROUNDED" and (.decision_search.safe_next_checks | length) == 0 and (.decision_search.missing_evidence | length) == 0 and .claim_boundary.network_scanner_ready == false' <<<"$big_linux_decision_search_snapshot_json" >/dev/null
big_linux_relation_profile_json="$("$llmwave_big" linux-relation-profile --residual-pack "$tmp_linux_chat_residual" --format json)"
jq -e '.mode == "llmwave-big-linux-relation-profile" and .verdict == "LINUX_RELATION_PROFILE_READY_NOT_CORPUS_COMPLETE" and (.causal_chains[] | select(.chain_id == "vulnerability_boundary" and .present == true)) and .claim_boundary.general_llm_ready == false' <<<"$big_linux_relation_profile_json" >/dev/null
big_security_fixture_json="$("$llmwave_big" security-fixture-run --format json)"
jq -e '.mode == "llmwave-big-security-fixture-run" and .verdict == "DEFENSIVE_PATCH_PROVEN_LOCAL_FIXTURE" and .finding.class == "PATH_TRAVERSAL_RISK" and .result.finding_found == true and .result.patch_candidate_generated == true and .result.before_exploit_reaches_forbidden_path == true and .result.after_forbidden_path_blocked == true and .result.regression_normal_file_still_reads == true and .verification.all_passed == true and .claim_boundary.real_project_scanner_ready == false' <<<"$big_security_fixture_json" >/dev/null
big_daybreak_duel_json="$("$llmwave_big" daybreak-duel --format json)"
jq -e '.mode == "llmwave-big-daybreak-duel" and .verdict == "DAYBREAK_DUEL_BASELINE_READY_NOT_COMPETITIVE" and .scoreboard.total == 6 and .scoreboard.passed == 5 and .scoreboard.blocked == 1 and .claim_boundary.defensive_duel_ready == true and .claim_boundary.runtime_snapshot_overlay_ready == true and .claim_boundary.patch_generation_ready == false and .claim_boundary.daybreak_competitive == false' <<<"$big_daybreak_duel_json" >/dev/null
jq -e '.challenges[] | select(.id == "runtime_snapshot_firewall_overlay" and .state == "PASS")' <<<"$big_daybreak_duel_json" >/dev/null
jq -e '.challenges[] | select(.id == "local_patch_fixture_loop" and .state == "PASS")' <<<"$big_daybreak_duel_json" >/dev/null
jq -e '.challenges[] | select(.id == "real_project_remediation_verification" and .state == "BLOCKED")' <<<"$big_daybreak_duel_json" >/dev/null
big_small_domain_claim_json="$("$llmwave_big" claim-gate --claim small-domain-llmwave --format json)"
jq -e '.claim == "small-domain-llmwave" and .verdict == "CLAIM_ALLOWED_LOCAL_ONLY" and .allowed == true and (.missing_evidence | index("broad unscripted chat eval"))' <<<"$big_small_domain_claim_json" >/dev/null
set +e
big_llm_claim_json="$("$llmwave_big" claim-gate --claim llm-ready --format json)"
big_llm_claim_code=$?
set -e
test "$big_llm_claim_code" -eq 3
jq -e '.claim == "llm-ready" and .verdict == "CLAIM_BLOCKED" and .allowed == false and (.missing_evidence | index("broad multi-turn chat eval"))' <<<"$big_llm_claim_json" >/dev/null
set +e
big_nonlinear_claim_json="$("$llmwave_big" claim-gate --claim nonlinear-memory --format json)"
big_nonlinear_claim_code=$?
set -e
test "$big_nonlinear_claim_code" -eq 3
jq -e '.claim == "nonlinear-memory" and .verdict == "CLAIM_BLOCKED" and .allowed == false and (.missing_evidence | index("fixed-basis beats linear baseline under capacity"))' <<<"$big_nonlinear_claim_json" >/dev/null
big_nonlinear_eval_json="$("$llmwave_big" nonlinear-memory-eval --format json)"
jq -e '.mode == "llmwave-big-nonlinear-memory-eval" and .verdict == "NONLINEAR_MEMORY_SCALE_CANDIDATE_NOT_PROVEN" and .aggregate.state == "USEFUL_DENSITY_SCALE_CANDIDATE"' <<<"$big_nonlinear_eval_json" >/dev/null
jq -e '.basis.fixed_across_sweep == true and .basis.wave_dim == 1024 and (.sweep | length) == 5 and .sweep[-1].facts == 15000 and .sweep[-1].verdict == "WAVE_DENSITY_WIN"' <<<"$big_nonlinear_eval_json" >/dev/null
jq -e '.aggregate.large_scale_win_rate == 1 and .aggregate.large_scale_bytes_per_useful_fact_gain > 4 and .aggregate.max_role_error_rate <= 0.02 and .aggregate.max_false_positive_rate <= 0.02' <<<"$big_nonlinear_eval_json" >/dev/null
jq -e '.claim_boundary.nonlinear_memory_eval_implemented == true and .claim_boundary.useful_density_candidate == true and .claim_boundary.nonlinear_memory_proven == false and (.claim_boundary.blocked_by | index("external_corpus_missing")) and (.claim_boundary.blocked_by | index("broad_noise_eval_missing"))' <<<"$big_nonlinear_eval_json" >/dev/null
big_nonlinear_ladder_json="$("$llmwave_big" nonlinear-memory-ladder --max-facts 100000 --format json)"
jq -e '.mode == "llmwave-big-nonlinear-memory-ladder" and .phase == "phase-1-nonlinear-memory-ladder" and .verdict == "PHASE1_DENSITY_LADDER_READY" and (.ladder | length) == 5' <<<"$big_nonlinear_ladder_json" >/dev/null
jq -e '.ladder[0].facts == 10 and .ladder[-1].facts == 100000 and .aggregate.phase1_ready == true' <<<"$big_nonlinear_ladder_json" >/dev/null
jq -e '.aggregate.amortized_win_point != null and .aggregate.standalone_break_even_point != null and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.final_proof_gate_passed == false' <<<"$big_nonlinear_ladder_json" >/dev/null
jq -e '([.ladder[].verdict] | index("AMORTIZED_WAVE_WIN") or index("STANDALONE_BASIS_REPAID"))' <<<"$big_nonlinear_ladder_json" >/dev/null
big_nonlinear_fixture_json="$("$llmwave_big" nonlinear-memory-eval --corpus "$root/examples/llmwave-big-nonlinear-memory-corpus.json" --format json)"
jq -e '.external_corpus.state == "EXTERNAL_FIXTURE_AND_NOISE_PASS" and .external_corpus.heldout_pass_rate == 1 and .external_corpus.negative_reject_rate == 1 and .external_corpus.noise_reject_rate == 1' <<<"$big_nonlinear_fixture_json" >/dev/null
jq -e '.corpus_driven_memory.verdict == "CORPUS_DRIVEN_AMORTIZED_DENSITY_OBSERVED" and .corpus_driven_memory.gates.corpus_driven_nonlinear_density_observed == true and .corpus_driven_memory.gates.strict_standalone_density_observed == false' <<<"$big_nonlinear_fixture_json" >/dev/null
jq -e '.corpus_driven_memory.delta.amortized_bytes_per_useful_fact_gain > 1.2 and .corpus_driven_memory.delta.standalone_bytes_per_useful_fact_gain < 1 and .corpus_driven_memory.fixed_basis_amortized.bytes_total < .corpus_driven_memory.linear_baseline.bytes_total' <<<"$big_nonlinear_fixture_json" >/dev/null
jq -e '.proof_policy.selected == "strict-full-sweep" and .proof_policy.selected_policy_proven == false and .proof_policy.scale_amortized_nonlinear_memory_proven == true and .claim_boundary.scale_amortized_nonlinear_memory_proven == true and .claim_boundary.nonlinear_memory_proven == false and (.claim_boundary.blocked_by | index("external_corpus_missing") | not) and (.claim_boundary.blocked_by | index("broad_noise_eval_missing") | not) and (.claim_boundary.blocked_by | index("fixed_basis_does_not_beat_linear_baseline"))' <<<"$big_nonlinear_fixture_json" >/dev/null
big_nonlinear_scale_policy_json="$("$llmwave_big" nonlinear-memory-eval --corpus "$root/examples/llmwave-big-nonlinear-memory-corpus.json" --proof-policy scale-amortized --format json)"
jq -e '.proof_policy.selected == "scale-amortized" and .proof_policy.selected_policy_proven == true and .proof_policy.general_claim_unlocked == false and .claim_boundary.selected_policy == "scale-amortized" and .claim_boundary.selected_policy_proven == true and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_nonlinear_scale_policy_json" >/dev/null
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
big_surface_reconstruct_json="$("$llmwave_big" surface-reconstruct --format json)"
jq -e '.roadmap_block == "v261-v270" and .verdict == "SURFACE_RECONSTRUCT_READY"' <<<"$big_surface_reconstruct_json" >/dev/null
jq -e '.eval.cases == 4 and .eval.exact_matches == 4 and .eval.exact_match_rate == 1 and .eval.state == "TOY_RECONSTRUCTION_PASS_NOT_DENSITY_PROOF"' <<<"$big_surface_reconstruct_json" >/dev/null
jq -e '([.cases[] | "\(.path):\(.reconstructed)"] | index("surface_program:invoice") and index("surface_program:invoicing") and index("evidence_copy_span:PI-HL-RLTG-GZ-20260611-03") and index("byte_fallback:zxq"))' <<<"$big_surface_reconstruct_json" >/dev/null
jq -e '.bank_summary.hot_core_contains_utf8 == false and .claim_boundary.hot_core_utf8_free == true and .claim_boundary.real_corpus_trained == false and .claim_boundary.free_form_spelling_proven == false and .claim_boundary.nonlinear_surface_memory_proven == false' <<<"$big_surface_reconstruct_json" >/dev/null
big_surface_corpus_json="$("$llmwave_big" surface-corpus-eval --format json)"
jq -e '.roadmap_block == "v271-v280" and .verdict == "SURFACE_DENSITY_CANDIDATE_NOT_PROVEN"' <<<"$big_surface_corpus_json" >/dev/null
jq -e '.corpus.productive_forms == 512 and .reconstruction.exact_match_rate == 1 and .reconstruction.held_out_exact_match_rate == 1' <<<"$big_surface_corpus_json" >/dev/null
jq -e '.baselines.family_template_bytes < .baselines.direct_lookup_bytes and .baselines.family_template_bytes < .baselines.per_form_program_bytes and .baselines.family_vs_direct_saving_ratio > 0' <<<"$big_surface_corpus_json" >/dev/null
jq -e '.family_reuse.state == "FAMILY_REUSE_VISIBLE" and .verdict_boundary.useful_density_candidate == true and .verdict_boundary.nonlinear_surface_memory_proven == false and .verdict_boundary.real_corpus_trained == false' <<<"$big_surface_corpus_json" >/dev/null
big_surface_bank_json="$("$llmwave_big" surface-bank-build --format json)"
jq -e '.roadmap_block == "v281-v290" and .verdict == "SURFACE_BANK_BUILD_READY_NOT_REAL_TRAINING"' <<<"$big_surface_bank_json" >/dev/null
jq -e '.bank_summary.accepted_family_count == 3 and .eval.held_out_exact_match_rate == 1 and .eval.state == "OBSERVED_BANK_BUILD_PASS_NOT_DENSITY_PROOF"' <<<"$big_surface_bank_json" >/dev/null
jq -e '([.accepted_families[].held_out_reconstructions[]] | index("invoicing") and index("customing") and index("routing"))' <<<"$big_surface_bank_json" >/dev/null
jq -e '([.rejected_fragments[].path] | index("evidence_copy_span")) and .claim_boundary.useful_density_candidate == true and .claim_boundary.real_corpus_trained == false and .claim_boundary.nonlinear_surface_memory_proven == false' <<<"$big_surface_bank_json" >/dev/null
big_surface_bank_validate_json="$("$llmwave_big" surface-bank-validate --format json)"
jq -e '.roadmap_block == "v291-v300" and .verdict == "SURFACE_BANK_VALIDATE_READY_NOT_REAL_TRAINING"' <<<"$big_surface_bank_validate_json" >/dev/null
jq -e '.metrics.positive_accept_rate == 1 and .metrics.negative_reject_rate == 1 and .metrics.shuffle_stability_rate == 1 and .metrics.false_family_rate == 0' <<<"$big_surface_bank_validate_json" >/dev/null
jq -e '([.negative_controls[] | select(.accepted == false) | .case_id] | index("invoiceing_trap") and index("rare_code_family_trap") and index("short_root_trap"))' <<<"$big_surface_bank_validate_json" >/dev/null
jq -e '.shuffle_stability.state == "ORDER_STABLE_ON_EMBEDDED_CORPUS" and .claim_boundary.validation_passed == true and .claim_boundary.real_corpus_trained == false and .claim_boundary.nonlinear_surface_memory_proven == false' <<<"$big_surface_bank_validate_json" >/dev/null
big_surface_bank_fixture_json="$("$llmwave_big" surface-bank-fixture --corpus "$root/examples/llmwave-big-surface-corpus.json" --format json)"
jq -e '.roadmap_block == "v301-v310" and .verdict == "SURFACE_BANK_FIXTURE_READY_NOT_REAL_TRAINING"' <<<"$big_surface_bank_fixture_json" >/dev/null
jq -e '.corpus.family_count == 6 and .corpus.held_out_forms == 6 and .corpus.negative_controls == 6 and .metrics.fixture_loaded == true' <<<"$big_surface_bank_fixture_json" >/dev/null
jq -e '.metrics.positive_exact_match_rate == 1 and .metrics.negative_reject_rate == 1 and .metrics.rare_copy_span_rate == 1 and .metrics.false_family_rate == 0' <<<"$big_surface_bank_fixture_json" >/dev/null
jq -e '.metrics.state == "EXTERNAL_FIXTURE_PASS_NOT_GENERAL_PROOF" and .claim_boundary.external_fixture_loaded == true and .claim_boundary.real_corpus_trained == false and .claim_boundary.nonlinear_surface_memory_proven == false' <<<"$big_surface_bank_fixture_json" >/dev/null
big_surface_bank_ru_json="$("$llmwave_big" surface-bank-fixture --corpus "$root/examples/llmwave-big-surface-corpus-ru.json" --format json)"
jq -e '.roadmap_block == "v301-v310" and .verdict == "SURFACE_BANK_FIXTURE_READY_NOT_REAL_TRAINING"' <<<"$big_surface_bank_ru_json" >/dev/null
jq -e '.corpus.source == "russian_business_surface_fixture_v1" and .corpus.family_count == 6 and .corpus.rare_forms == 3' <<<"$big_surface_bank_ru_json" >/dev/null
jq -e '.metrics.positive_exact_match_rate == 1 and .metrics.negative_reject_rate == 1 and .metrics.rare_copy_span_rate == 1 and .claim_boundary.nonlinear_surface_memory_proven == false' <<<"$big_surface_bank_ru_json" >/dev/null
big_surface_raw_ru_json="$("$llmwave_big" surface-raw-induce --corpus "$root/examples/llmwave-big-raw-surface-corpus-ru.json" --format json)"
jq -e '.roadmap_block == "v311-v320" and .verdict == "SURFACE_RAW_INDUCE_READY_NOT_REAL_TRAINING"' <<<"$big_surface_raw_ru_json" >/dev/null
jq -e '.corpus.source == "russian_raw_business_surface_fixture_v1" and .metrics.induced_family_count == 6 and .claim_boundary.roots_given_to_inducer == false' <<<"$big_surface_raw_ru_json" >/dev/null
jq -e '.metrics.expected_root_recall == 1 and .metrics.held_out_exact_match_rate == 1 and .metrics.negative_reject_rate == 1 and .metrics.false_family_rate == 0' <<<"$big_surface_raw_ru_json" >/dev/null
big_surface_raw_noisy_json="$("$llmwave_big" surface-raw-induce --corpus "$root/examples/llmwave-big-raw-surface-corpus-ru-noisy.json" --format json)"
jq -e '.roadmap_block == "v321-v330" and .corpus.source == "russian_noisy_raw_business_surface_fixture_v1"' <<<"$big_surface_raw_noisy_json" >/dev/null
jq -e '.metrics.induced_family_count == 6 and .metrics.expected_root_recall == 1 and .metrics.noise_reject_rate == 1 and .metrics.false_family_rate == 0' <<<"$big_surface_raw_noisy_json" >/dev/null
jq -e '([.rejected_collision_roots[].root] | index("счетчик") and index("договоренност") and index("маршрутизатор") and index("сертификатор")) and .metrics.state == "NOISY_RAW_INDUCTION_PASS_NOT_GENERAL_PROOF"' <<<"$big_surface_raw_noisy_json" >/dev/null
big_surface_raw_derived_json="$("$llmwave_big" surface-raw-induce --corpus "$root/examples/llmwave-big-raw-surface-corpus-ru-derived.json" --format json)"
jq -e '.roadmap_block == "v331-v360" and .corpus.source == "russian_derived_suffix_raw_business_surface_fixture_v1"' <<<"$big_surface_raw_derived_json" >/dev/null
jq -e '.corpus.suffix_inventory_source == "derived_from_raw_forms" and .derived_suffix_inventory.enabled == true and .metrics.manual_suffix_count == 0 and .metrics.derived_suffix_count >= 8' <<<"$big_surface_raw_derived_json" >/dev/null
jq -e '.metrics.induced_family_count == 9 and .metrics.expected_root_recall == 1 and .metrics.held_out_exact_match_rate == 1 and .metrics.noise_reject_rate == 1 and .metrics.false_family_rate == 0' <<<"$big_surface_raw_derived_json" >/dev/null
jq -e '([.induced_families[].root] | index("деклараци") and index("инструкци") and index("счет")) and ([.rejected_collision_roots[].root] | index("счетчик") and index("маршрутизатор")) and .metrics.state == "DERIVED_SUFFIX_RAW_INDUCTION_PASS_NOT_GENERAL_PROOF"' <<<"$big_surface_raw_derived_json" >/dev/null
tmp_big_train="$(mktemp -d)"
cat > "$tmp_big_train/corpus.txt" <<'EOF'
Honglu issues invoice. invoice requires payment.
Payment supports customs declaration. declaration requires evidence.
Honglu issues invoice. invoice requires payment.
evidence blocks unsupported answer.
EOF
big_train_json="$("$llmwave_big" train "$tmp_big_train/corpus.txt" --out "$tmp_big_train/artifact.json" --vocab-cap 128 --transition-cap 256 --active-chunk-cap 64 --chunk-tokens 8 --extensions txt --format json)"
jq -e '.version == "llmwave-big-v1901-corpus-training" and .verdict == "TRAINING_ARTIFACT_READY_NOT_LLM" and .claim_boundary.real_corpus_loaded == true and .claim_boundary.chat_llm_ready == false' <<<"$big_train_json" >/dev/null
jq -e '.field_budget.fits_hot_budget == true and .field_budget.transition_records > 0 and .eval.state == "HELD_OUT_EVAL_RAN"' <<<"$big_train_json" >/dev/null
test -s "$tmp_big_train/artifact.json"
big_ask_json="$("$llmwave_big" ask --artifact "$tmp_big_train/artifact.json" --text "what requires evidence" --top-k 3 --format json)"
jq -e '.version == "llmwave-big-v1902-artifact-ask" and .claim_boundary.artifact_loaded == true and .claim_boundary.trained_field_used == true and .claim_boundary.broad_chat_llm_ready == false' <<<"$big_ask_json" >/dev/null
jq -e '.field.top_chunk_peaks | length > 0' <<<"$big_ask_json" >/dev/null
cat > "$tmp_big_train/ask-eval.json" <<'EOF'
{"cases":[
  {"id":"requires","query":"what does invoice require","expected_contains":"invoice requires payment","expected_safe_to_answer":true},
  {"id":"unknown","query":"moonlight customs route","expected_contains":"","expected_safe_to_answer":false}
]}
EOF
big_ask_eval_json="$("$llmwave_big" ask-eval --artifact "$tmp_big_train/artifact.json" --suite "$tmp_big_train/ask-eval.json" --top-k 3 --format json)"
jq -e '.version == "llmwave-big-v1903-artifact-ask-eval" and .verdict == "ARTIFACT_ASK_EVAL_PASS_NOT_GENERAL_LLM" and .metrics.false_positive_rate == 0' <<<"$big_ask_eval_json" >/dev/null
big_hot_pack_json="$("$llmwave_big" pack-hot --artifact "$tmp_big_train/artifact.json" --out "$tmp_big_train/artifact.hot.bin" --format json)"
jq -e '.version == "llmwave-big-v1904-hot-pack" and .verdict == "HOT_PACK_READY_NOT_CACHE_ONLY_PROOF" and .bytes.fits_hot_budget == true' <<<"$big_hot_pack_json" >/dev/null
jq -e '.claim_boundary.binary_hot_pack_written == true and .claim_boundary.strings_excluded_from_hot_pack == true and .claim_boundary.json_excluded_from_hot_pack == true and .claim_boundary.cache_only_execution_proven == false' <<<"$big_hot_pack_json" >/dev/null
test -s "$tmp_big_train/artifact.hot.bin"
big_ask_hot_json="$("$llmwave_big" ask-hot --hot-pack "$tmp_big_train/artifact.hot.bin" --artifact "$tmp_big_train/artifact.json" --text "invoice requires payment" --top-k 3 --format json)"
jq -e '.version == "llmwave-big-v1905-hot-ask" and .verdict == "HOT_FIELD_ANSWER_READY_NOT_GENERAL_LLM" and .field.state == "HOT_FIELD_SCHEMA_FOCUSED" and .answer.safe_to_answer == true' <<<"$big_ask_hot_json" >/dev/null
jq -e '.field.polarity_state == "ALIGNED" and .field.top_hot_schema_peaks[0].polarization.state == "ALIGNED" and .field.top_hot_schema_peaks[0].polarization.hard_stop == false' <<<"$big_ask_hot_json" >/dev/null
jq -e '.claim_boundary.binary_hot_pack_loaded == true and .claim_boundary.hot_records_scanned == true and .claim_boundary.polarity_lens_applied == true and .claim_boundary.reversed_polarity_hard_stop == true and .claim_boundary.cold_artifact_used_for_labels == true and .claim_boundary.json_used_in_hot_scan == false and .claim_boundary.cache_only_execution_proven == false' <<<"$big_ask_hot_json" >/dev/null
big_ask_hot_reversed_json="$("$llmwave_big" ask-hot --hot-pack "$tmp_big_train/artifact.hot.bin" --artifact "$tmp_big_train/artifact.json" --text "payment requires invoice" --top-k 3 --format json)"
jq -e '.version == "llmwave-big-v1905-hot-ask" and .verdict == "HOT_FIELD_REVIEW" and .field.state == "HOT_FIELD_POLARITY_REVERSED" and .answer.safe_to_answer == false' <<<"$big_ask_hot_reversed_json" >/dev/null
jq -e '.field.polarity_state == "REVERSED" and .field.top_hot_schema_peaks[0].polarization.state == "REVERSED" and .field.top_hot_schema_peaks[0].polarization.hard_stop == true and .answer.state == "HOT_POLARITY_REVERSED_STOP"' <<<"$big_ask_hot_reversed_json" >/dev/null
big_ask_hot_before_learn_json="$("$llmwave_big" ask-hot --hot-pack "$tmp_big_train/artifact.hot.bin" --artifact "$tmp_big_train/artifact.json" --text "supplier requires customs" --top-k 3 --format json)"
jq -e '.version == "llmwave-big-v1905-hot-ask" and .answer.safe_to_answer == false and .learning.memory_loaded == false' <<<"$big_ask_hot_before_learn_json" >/dev/null
cat > "$tmp_big_train/hot-feedback.json" <<'EOF'
{"events":[
  {"decision":"accept","subject":"supplier","relation":"requires","object":"customs","authority":1.0,"source":"user-batch"}
]}
EOF
big_learn_hot_json="$("$llmwave_big" learn-hot --feedback "$tmp_big_train/hot-feedback.json" --out "$tmp_big_train/hot-memory.json" --format json)"
jq -e '.version == "llmwave-big-v1906-hot-learning-memory" and .verdict == "HOT_LEARNING_MEMORY_WRITTEN_NOT_GRADIENT_TRAINING" and .memory.records_written == 1 and .claim_boundary.can_change_next_hot_ask == true' <<<"$big_learn_hot_json" >/dev/null
big_ask_hot_after_learn_json="$("$llmwave_big" ask-hot --hot-pack "$tmp_big_train/artifact.hot.bin" --artifact "$tmp_big_train/artifact.json" --memory "$tmp_big_train/hot-memory.json" --text "supplier requires customs" --top-k 3 --format json)"
jq -e '.version == "llmwave-big-v1905-hot-ask" and .verdict == "HOT_FIELD_ANSWER_READY_NOT_GENERAL_LLM" and .answer.safe_to_answer == true and .learning.memory_loaded == true and .learning.learned_records == 1' <<<"$big_ask_hot_after_learn_json" >/dev/null
jq -e '.field.top_hot_schema_peaks[0].learned == true and .field.top_hot_schema_peaks[0].source == "hot_memory:user-batch" and .field.top_hot_schema_peaks[0].subject == "supplier" and .field.top_hot_schema_peaks[0].object == "customs"' <<<"$big_ask_hot_after_learn_json" >/dev/null
cat > "$tmp_big_train/chat.script" <<'EOF'
ask broker requires invoice
learn accept: broker | requires | invoice
ask broker requires invoice
exit
EOF
big_chat_hot_json="$("$llmwave_big" chat-hot --hot-pack "$tmp_big_train/artifact.hot.bin" --artifact "$tmp_big_train/artifact.json" --memory "$tmp_big_train/chat-memory.json" --script "$tmp_big_train/chat.script" --top-k 3 --format json)"
jq -e '.version == "llmwave-big-v1907-hot-chat" and .verdict == "HOT_CHAT_SESSION_READY_NOT_GENERAL_LLM" and (.turns | length) == 4' <<<"$big_chat_hot_json" >/dev/null
jq -e '.turns[0].kind == "ask" and .turns[0].safe_to_answer == false and .turns[1].kind == "learn" and .turns[1].state == "HOT_MEMORY_UPDATED" and .turns[2].kind == "ask" and .turns[2].safe_to_answer == true' <<<"$big_chat_hot_json" >/dev/null
test -s "$tmp_big_train/chat-memory.json"
big_chat_hot_eval_json="$("$llmwave_big" chat-hot-eval --hot-pack "$tmp_big_train/artifact.hot.bin" --artifact "$tmp_big_train/artifact.json" --memory "$tmp_big_train/chat-eval-memory.json" --script "$tmp_big_train/chat.script" --top-k 3 --format json)"
jq -e '.version == "llmwave-big-v1908-hot-chat-eval" and .verdict == "HOT_CHAT_EVAL_PASS_NOT_GENERAL_LLM" and .metrics.memory_lift_observed == true and .metrics.false_safe_before_learning == false and .metrics.pass_rate == 1' <<<"$big_chat_hot_eval_json" >/dev/null
jq -e '.claim_boundary.scripted_hot_chat_eval_implemented == true and .claim_boundary.multi_turn_memory_lift_observed == true and .claim_boundary.broad_chat_llm_ready == false and .claim_boundary.general_dialogue_ready == false' <<<"$big_chat_hot_eval_json" >/dev/null
test -s "$tmp_big_train/chat-eval-memory.json"
big_domain_eval_json="$("$llmwave_big" domain-eval --artifact "$tmp_big_train/artifact.json" --ask-suite "$tmp_big_train/ask-eval.json" --hot-pack "$tmp_big_train/artifact.hot.bin" --chat-script "$tmp_big_train/chat.script" --chat-memory "$tmp_big_train/domain-chat-memory.json" --nonlinear-corpus "$root/examples/llmwave-big-nonlinear-memory-corpus.json" --top-k 3 --format json)"
jq -e '.version == "llmwave-big-v1909-domain-eval" and .verdict == "SMALL_DOMAIN_LLMWAVE_EVAL_PASS_NOT_BROAD_LLM" and .metrics.passed_components == 3 and .metrics.pass_rate == 1' <<<"$big_domain_eval_json" >/dev/null
jq -e '.claim_boundary.small_domain_llmwave_ready == true and .claim_boundary.artifact_grounded_qa_ready == true and .claim_boundary.scripted_hot_multi_turn_ready == true and .claim_boundary.scale_amortized_nonlinear_memory_ready == true and .claim_boundary.broad_chat_llm_ready == false and .claim_boundary.general_llm_ready == false' <<<"$big_domain_eval_json" >/dev/null
test -s "$tmp_big_train/domain-chat-memory.json"
rm -rf "$tmp_big_train"
tmp_big_demo="$(mktemp -d)"
big_demo_domain_json="$("$llmwave_big" demo-domain --out-dir "$tmp_big_demo/demo" --nonlinear-corpus "$root/examples/llmwave-big-nonlinear-memory-corpus.json" --format json)"
jq -e '.version == "llmwave-big-v1910-demo-domain" and .verdict == "DEMO_DOMAIN_PASS_NOT_BROAD_LLM" and .metrics.passed_components == 5 and .metrics.pass_rate == 1' <<<"$big_demo_domain_json" >/dev/null
jq -e '.claim_boundary.demo_domain_command_ready == true and .claim_boundary.small_domain_llmwave_ready == true and .claim_boundary.scripted_hot_multi_turn_ready == true and .claim_boundary.scale_amortized_nonlinear_memory_ready == true and .claim_boundary.broad_chat_llm_ready == false and .claim_boundary.general_llm_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_demo_domain_json" >/dev/null
jq -e '.steps.training.passed == true and .steps.hot_pack.passed == true and .steps.hot_chat_eval.memory_lift_observed == true and .steps.nonlinear_memory_eval.selected_policy_proven == true and .steps.domain_eval.verdict == "SMALL_DOMAIN_LLMWAVE_EVAL_PASS_NOT_BROAD_LLM"' <<<"$big_demo_domain_json" >/dev/null
test -s "$tmp_big_demo/demo/project-artifact.json"
test -s "$tmp_big_demo/demo/project.hot.bin"
rm -rf "$tmp_big_demo"
big_write_json="$("$llmwave_big" write --format json)"
jq -e '.roadmap_block == "v191-v205" and .verdict == "RESIDUAL_SAVING"' <<<"$big_write_json" >/dev/null
jq -e '.residual_format_v1.bytes == 20 and .write_decision.bytes_written == 28' <<<"$big_write_json" >/dev/null
jq -e '.write_curve.state == "SYNTHETIC_CONTRACT_CURVE_NOT_NONLINEAR_PROOF" and .write_curve.residual_saving_ratio > 0.5' <<<"$big_write_json" >/dev/null
jq -e '.compression_safety.safe == true and .anti_residual.anti_lane_id == 90001' <<<"$big_write_json" >/dev/null
big_schema_residual_engine_json="$("$llmwave_big" schema-residual-engine --format json)"
jq -e '.mode == "llmwave-big-schema-residual-engine" and .phase == "phase-2-3-schema-reuse-residual-write" and .verdict == "PHASE2_3_SCHEMA_RESIDUAL_ENGINE_READY"' <<<"$big_schema_residual_engine_json" >/dev/null
jq -e '.promoted_schema_count == 3 and .residual_write_count == 10 and .full_fallback_count == 1 and .metrics.bytes_per_useful_fact_gain > 2' <<<"$big_schema_residual_engine_json" >/dev/null
jq -e '.claim_boundary.schema_reuse_engine_implemented == true and .claim_boundary.residual_only_write_implemented == true and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_schema_residual_engine_json" >/dev/null
big_memory_physics_json="$("$llmwave_big" memory-physics --format json)"
jq -e '.mode == "llmwave-big-memory-physics" and .phase == "phase-4-5-collision-noise-anti-wave" and .verdict == "PHASE4_5_MEMORY_PHYSICS_READY"' <<<"$big_memory_physics_json" >/dev/null
jq -e '.schema_residual_bridge.phase4_5_uses_phase2_3_engine == true and .anti_wave_format.record_bytes == 32 and .anti_wave_format.shortcut_specific == true' <<<"$big_memory_physics_json" >/dev/null
jq -e '.metrics.collision_reject_rate == 1 and .metrics.noise_reject_rate == 1 and .metrics.false_positive_rate_before_anti > .metrics.false_positive_rate_after_anti and .metrics.false_positive_rate_after_anti == 0' <<<"$big_memory_physics_json" >/dev/null
jq -e '.claim_boundary.collision_noise_physics_implemented == true and .claim_boundary.anti_wave_memory_integrated == true and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_memory_physics_json" >/dev/null
big_memory_proof_path_json="$("$llmwave_big" memory-proof-path --format json)"
jq -e '.mode == "llmwave-big-memory-proof-path" and .phase == "phase-6-8-heldout-basis-atlas" and .verdict == "PHASE6_8_MEMORY_PROOF_PATH_READY"' <<<"$big_memory_proof_path_json" >/dev/null
jq -e '.metrics.heldout_pass_rate == 1 and .metrics.memory_physics_ready == true and .wave_atlas.route_balanced == true and .basis_economics.ladder_phase_ready == true' <<<"$big_memory_proof_path_json" >/dev/null
jq -e '.claim_boundary.heldout_inference_implemented == true and .claim_boundary.basis_economics_connected == true and .claim_boundary.wave_atlas_memory_implemented == true and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_memory_proof_path_json" >/dev/null
big_memory_final_proof_json="$("$llmwave_big" memory-final-proof --format json)"
jq -e '.mode == "llmwave-big-memory-final-proof" and .phase == "phase-9-12-field-recall-llmwave-big-corpus-final-proof" and .verdict == "FINAL_PROOF_GATE_BLOCKED_BY_BIG_CORPUS"' <<<"$big_memory_final_proof_json" >/dev/null
jq -e '.final_proof_gate.controlled_chain_ready == true and .final_proof_gate.field_recall_ready == true and .final_proof_gate.llmwave_bridge_ready == true and .final_proof_gate.big_corpus_ready == false' <<<"$big_memory_final_proof_json" >/dev/null
jq -e '.final_proof_gate.final_proof_gate_passed == false and .final_proof_gate.nonlinear_memory_proven == false and .final_proof_gate.llm_ready == false and (.final_proof_gate.missing_evidence | index("real_or_profile_big_corpus_not_loaded"))' <<<"$big_memory_final_proof_json" >/dev/null
jq -e '.claim_boundary.phases_1_12_command_path_implemented == true and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_memory_final_proof_json" >/dev/null
big_memory_final_proof_rust_json="$("$llmwave_big" memory-final-proof --profile rust --format json)"
jq -e '.profile == "rust" and .big_corpus_gate.corpus_kind == "rust-code-structural-corpus" and .field_recall.dominant_route == "rust-cli-proof-route"' <<<"$big_memory_final_proof_rust_json" >/dev/null
jq -e '.rust_profile != null and (.rust_profile.target_routes | index("cli-command-dispatch")) and (.rust_profile.forbidden_shortcuts | index("compiled command implies LLM readiness"))' <<<"$big_memory_final_proof_rust_json" >/dev/null
jq -e '.verdict == "FINAL_PROOF_GATE_BLOCKED_BY_BIG_CORPUS" and (.big_corpus_gate.blocked_by | index("no_rust_code_corpus_artifact")) and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_memory_final_proof_rust_json" >/dev/null
tmp_rust_corpus="$(mktemp -d)"
big_rust_corpus_json="$("$llmwave_big" rust-corpus-build --repo "$root" --out "$tmp_rust_corpus/rust-corpus.json" --format json)"
jq -e '.mode == "llmwave-big-rust-corpus-build" and .profile == "rust" and .verdict == "RUST_CORPUS_ARTIFACT_READY"' <<<"$big_rust_corpus_json" >/dev/null
jq -e '.artifact.corpus_kind == "rust-code-structural-corpus" and .artifact.rust_files > 10 and .artifact.functions > 100 and .artifact.fact_count > .artifact.rust_files' <<<"$big_rust_corpus_json" >/dev/null
jq -e '.claim_boundary.rust_corpus_loaded == true and .claim_boundary.heldout_suite_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_rust_corpus_json" >/dev/null
test -s "$tmp_rust_corpus/rust-corpus.json"
big_rust_heldout_json="$("$llmwave_big" rust-heldout-build --artifact "$tmp_rust_corpus/rust-corpus.json" --out "$tmp_rust_corpus/rust-heldout.json" --format json)"
jq -e '.mode == "llmwave-big-rust-heldout-build" and .profile == "rust" and .verdict == "RUST_HELDOUT_SUITE_READY"' <<<"$big_rust_heldout_json" >/dev/null
jq -e '.suite.suite_kind == "rust-code-heldout-suite" and .suite.heldout_case_count >= 16 and .suite.covered_routes >= 4 and .suite.negative_shortcut_count >= 3' <<<"$big_rust_heldout_json" >/dev/null
jq -e '.claim_boundary.rust_corpus_loaded == true and .claim_boundary.heldout_suite_ready == true and .claim_boundary.focus_packet_ready == false and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_rust_heldout_json" >/dev/null
test -s "$tmp_rust_corpus/rust-heldout.json"
big_rust_focus_json="$("$llmwave_big" rust-focus-build --artifact "$tmp_rust_corpus/rust-corpus.json" --heldout-suite "$tmp_rust_corpus/rust-heldout.json" --out "$tmp_rust_corpus/rust-focus.json" --format json)"
jq -e '.mode == "llmwave-big-rust-focus-build" and .profile == "rust" and .verdict == "RUST_FOCUS_PACKET_READY"' <<<"$big_rust_focus_json" >/dev/null
jq -e '.focus.packet_kind == "rust-code-route-balanced-focus" and .focus.selected_fact_count > 100 and .metrics.route_balance_after < .metrics.route_balance_before and .metrics.exact_withheld_facts_removed >= 16' <<<"$big_rust_focus_json" >/dev/null
jq -e '.claim_boundary.rust_corpus_loaded == true and .claim_boundary.heldout_suite_ready == true and .claim_boundary.focus_packet_ready == true and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_rust_focus_json" >/dev/null
test -s "$tmp_rust_corpus/rust-focus.json"
cat >"$tmp_rust_corpus/check.json" <<'JSON'
{"command":"cargo check --all-targets --all-features","exit_code":0,"stdout":"ok\n","stderr":""}
JSON
cat >"$tmp_rust_corpus/test.json" <<'JSON'
{"command":"scripts/test-local.sh","exit_code":0,"stdout":"ok\n","stderr":""}
JSON
cat >"$tmp_rust_corpus/clippy.json" <<'JSON'
{"command":"cargo clippy --all-targets --all-features -- -D warnings","exit_code":0,"stdout":"ok\n","stderr":""}
JSON
big_rust_compile_evidence_json="$("$llmwave_big" rust-compile-evidence-build --focus-packet "$tmp_rust_corpus/rust-focus.json" --check-evidence "$tmp_rust_corpus/check.json" --test-evidence "$tmp_rust_corpus/test.json" --clippy-evidence "$tmp_rust_corpus/clippy.json" --out "$tmp_rust_corpus/rust-compile-evidence.json" --format json)"
jq -e '.mode == "llmwave-big-rust-compile-evidence-build" and .profile == "rust" and .verdict == "RUST_COMPILE_EVIDENCE_READY"' <<<"$big_rust_compile_evidence_json" >/dev/null
jq -e '.evidence.compile_test_evidence_bridge_ready == true and .evidence.commands_passed == .evidence.commands_required and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_rust_compile_evidence_json" >/dev/null
test -s "$tmp_rust_corpus/rust-compile-evidence.json"
big_rust_heldout_eval_json="$("$llmwave_big" rust-heldout-eval --focus-packet "$tmp_rust_corpus/rust-focus.json" --heldout-suite "$tmp_rust_corpus/rust-heldout.json" --out "$tmp_rust_corpus/rust-heldout-eval.json" --format json)"
jq -e '.mode == "llmwave-big-rust-heldout-eval" and .profile == "rust" and .verdict == "RUST_HELDOUT_INFERENCE_EVAL_READY"' <<<"$big_rust_heldout_eval_json" >/dev/null
jq -e '.metrics.heldout_inference_eval_ready == true and .metrics.heldout_pass_rate >= 0.8 and .metrics.negative_reject_rate == 1 and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_rust_heldout_eval_json" >/dev/null
test -s "$tmp_rust_corpus/rust-heldout-eval.json"
big_memory_final_proof_rust_wired_json="$("$llmwave_big" memory-final-proof --profile rust --artifact "$tmp_rust_corpus/rust-corpus.json" --heldout-suite "$tmp_rust_corpus/rust-heldout.json" --focus-packet "$tmp_rust_corpus/rust-focus.json" --format json)"
jq -e '.verdict == "FINAL_PROOF_GATE_BLOCKED_BY_COMPILE_TEST_BRIDGE" and .big_corpus_gate.verdict == "PROFILE_CORPUS_FOCUS_READY_NOT_FINAL_PROOF"' <<<"$big_memory_final_proof_rust_wired_json" >/dev/null
jq -e '.big_corpus_gate.real_big_corpus_loaded == true and .big_corpus_gate.heldout_suite_ready == true and .big_corpus_gate.route_balanced_focus_ready == true and .final_proof_gate.compile_test_evidence_bridge_ready == false' <<<"$big_memory_final_proof_rust_wired_json" >/dev/null
jq -e '.claim_boundary.blocked_by == ["compile_test_evidence_bridge_missing"] and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_memory_final_proof_rust_wired_json" >/dev/null
big_memory_final_proof_rust_compile_json="$("$llmwave_big" memory-final-proof --profile rust --artifact "$tmp_rust_corpus/rust-corpus.json" --heldout-suite "$tmp_rust_corpus/rust-heldout.json" --focus-packet "$tmp_rust_corpus/rust-focus.json" --compile-evidence "$tmp_rust_corpus/rust-compile-evidence.json" --format json)"
jq -e '.verdict == "FINAL_PROOF_GATE_BLOCKED_BY_HELDOUT_INFERENCE_EVAL" and .big_corpus_gate.compile_test_evidence_bridge_ready == true' <<<"$big_memory_final_proof_rust_compile_json" >/dev/null
jq -e '.final_proof_gate.compile_test_evidence_bridge_ready == true and .final_proof_gate.heldout_inference_eval_ready == false and .claim_boundary.blocked_by == ["rust_heldout_inference_eval_missing"]' <<<"$big_memory_final_proof_rust_compile_json" >/dev/null
jq -e '.claim_boundary.nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_memory_final_proof_rust_compile_json" >/dev/null
big_memory_final_proof_rust_eval_json="$("$llmwave_big" memory-final-proof --profile rust --artifact "$tmp_rust_corpus/rust-corpus.json" --heldout-suite "$tmp_rust_corpus/rust-heldout.json" --focus-packet "$tmp_rust_corpus/rust-focus.json" --compile-evidence "$tmp_rust_corpus/rust-compile-evidence.json" --heldout-eval "$tmp_rust_corpus/rust-heldout-eval.json" --format json)"
jq -e '.verdict == "FINAL_PROOF_GATE_PROFILE_EVAL_READY_NOT_NONLINEAR_PROOF" and .final_proof_gate.profile_eval_ready == true' <<<"$big_memory_final_proof_rust_eval_json" >/dev/null
jq -e '.final_proof_gate.heldout_inference_eval_ready == true and .final_proof_gate.strict_nonlinear_density_claim_gate_ready == false and .claim_boundary.blocked_by == ["strict_nonlinear_density_claim_gate_missing"]' <<<"$big_memory_final_proof_rust_eval_json" >/dev/null
jq -e '.claim_boundary.nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_memory_final_proof_rust_eval_json" >/dev/null
big_strict_density_json="$("$llmwave_big" strict-density-claim-gate --artifact "$tmp_rust_corpus/rust-corpus.json" --focus-packet "$tmp_rust_corpus/rust-focus.json" --heldout-eval "$tmp_rust_corpus/rust-heldout-eval.json" --compile-evidence "$tmp_rust_corpus/rust-compile-evidence.json" --out "$tmp_rust_corpus/strict-density.json" --format json)"
jq -e '.mode == "llmwave-big-strict-density-claim-gate" and .profile == "rust" and .verdict == "STRICT_DENSITY_PROFILE_PROVEN"' <<<"$big_strict_density_json" >/dev/null
jq -e '.gates.rust_density_profile_proven == true and .gates.general_nonlinear_memory_proven == false and .density.density_win_ratio > 1 and .quality.heldout_pass_rate >= 0.9' <<<"$big_strict_density_json" >/dev/null
jq -e '.claim_boundary.rust_density_profile_proven == true and .claim_boundary.general_nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_strict_density_json" >/dev/null
test -s "$tmp_rust_corpus/strict-density.json"
big_memory_final_proof_rust_density_json="$("$llmwave_big" memory-final-proof --profile rust --artifact "$tmp_rust_corpus/rust-corpus.json" --heldout-suite "$tmp_rust_corpus/rust-heldout.json" --focus-packet "$tmp_rust_corpus/rust-focus.json" --compile-evidence "$tmp_rust_corpus/rust-compile-evidence.json" --heldout-eval "$tmp_rust_corpus/rust-heldout-eval.json" --strict-density-evidence "$tmp_rust_corpus/strict-density.json" --format json)"
jq -e '.verdict == "FINAL_PROOF_GATE_RUST_DENSITY_PROFILE_READY_NOT_GENERAL_LLM" and .final_proof_gate.rust_density_profile_ready == true' <<<"$big_memory_final_proof_rust_density_json" >/dev/null
jq -e '.final_proof_gate.final_proof_gate_passed == false and .final_proof_gate.general_nonlinear_memory_claim_ready == false and .claim_boundary.blocked_by == ["general_nonlinear_memory_multi_profile_eval_missing"]' <<<"$big_memory_final_proof_rust_density_json" >/dev/null
jq -e '.claim_boundary.nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_memory_final_proof_rust_density_json" >/dev/null
big_multi_profile_single_json="$("$llmwave_big" multi-profile-density-suite --rust-density "$tmp_rust_corpus/strict-density.json" --out "$tmp_rust_corpus/multi-single.json" --format json)"
jq -e '.mode == "llmwave-big-multi-profile-density-suite" and .verdict == "MULTI_PROFILE_DENSITY_BLOCKED_BY_SINGLE_PROFILE"' <<<"$big_multi_profile_single_json" >/dev/null
jq -e '.suite.profile_count == 1 and .suite.missing_profile_count == 2 and .gates.general_nonlinear_memory_proven == false' <<<"$big_multi_profile_single_json" >/dev/null
big_business_density_json="$("$llmwave_big" profile-density-build --profile business --corpus "$root/examples/llmwave-big-nonlinear-memory-corpus.json" --out "$tmp_rust_corpus/business-density.json" --format json)"
jq -e '.mode == "llmwave-big-profile-density-build" and .verdict == "PROFILE_DENSITY_PROVEN_NOT_GENERAL_LLM" and .gates.profile_density_proven == true and .gates.general_nonlinear_memory_proven == false' <<<"$big_business_density_json" >/dev/null
big_contracts_density_json="$("$llmwave_big" profile-density-build --profile contracts --corpus "$root/examples/llmwave-big-contract-density-corpus.json" --out "$tmp_rust_corpus/contracts-density.json" --format json)"
jq -e '.mode == "llmwave-big-profile-density-build" and .verdict == "PROFILE_DENSITY_PROVEN_NOT_GENERAL_LLM" and .gates.profile_density_proven == true and .gates.general_nonlinear_memory_proven == false' <<<"$big_contracts_density_json" >/dev/null
big_adversarial_density_json="$("$llmwave_big" profile-density-build --profile adversarial --corpus "$root/examples/llmwave-big-adversarial-density-corpus.json" --out "$tmp_rust_corpus/adversarial-density.json" --format json)"
jq -e '.mode == "llmwave-big-profile-density-build" and .verdict == "PROFILE_DENSITY_PROVEN_NOT_GENERAL_LLM" and .source.fact_count >= 64 and .quality.false_shortcut_rejection_rate == 1 and .quality.noise_reject_rate == 1' <<<"$big_adversarial_density_json" >/dev/null
big_multi_profile_pass_json="$("$llmwave_big" multi-profile-density-suite --rust-density "$tmp_rust_corpus/strict-density.json" --profile-evidence adversarial="$tmp_rust_corpus/adversarial-density.json" --profile-evidence contracts="$tmp_rust_corpus/contracts-density.json" --profile-evidence business="$tmp_rust_corpus/business-density.json" --out "$tmp_rust_corpus/multi-pass.json" --format json)"
jq -e '.verdict == "MULTI_PROFILE_NONLINEAR_MEMORY_PROVEN_NOT_LLM" and .suite.profile_count == 4 and .suite.passing_profile_count == 4' <<<"$big_multi_profile_pass_json" >/dev/null
jq -e '.gates.independent_profile_sources == true and .gates.general_nonlinear_memory_proven == true and .gates.llm_ready == false and .claim_boundary.llm_ready == false' <<<"$big_multi_profile_pass_json" >/dev/null
test -s "$tmp_rust_corpus/multi-pass.json"
big_multi_profile_duplicate_json="$("$llmwave_big" multi-profile-density-suite --rust-density "$tmp_rust_corpus/strict-density.json" --profile-evidence business-a="$tmp_rust_corpus/business-density.json" --profile-evidence business-b="$tmp_rust_corpus/business-density.json" --format json)"
jq -e '.verdict == "MULTI_PROFILE_DENSITY_BLOCKED" and .gates.independent_profile_sources == false and (.claim_boundary.blocked_by | index("duplicate_or_missing_independent_profile_sources"))' <<<"$big_multi_profile_duplicate_json" >/dev/null
big_density_doctor_json="$("$llmwave_big" density-proof-doctor --suite "$tmp_rust_corpus/multi-pass.json" --out "$tmp_rust_corpus/density-doctor.json" --format json)"
jq -e '.mode == "llmwave-big-density-proof-doctor" and .verdict == "DENSITY_PROOF_WEAK" and .gates.density_proof_doctor_medium_or_better == false and .gates.adversarial_profile_present == true and (.proof_quality.weak_spots | index("profile_corpora_too_small_or_unknown")) and (.proof_quality.weak_spots | index("adversarial_profile_missing") | not)' <<<"$big_density_doctor_json" >/dev/null
test -s "$tmp_rust_corpus/density-doctor.json"
jq -e '.verdict == "DENSITY_PROOF_WEAK" and .claim_boundary.general_nonlinear_memory_proven == false' "$tmp_rust_corpus/density-doctor.json" >/dev/null
big_memory_final_proof_multi_profile_json="$("$llmwave_big" memory-final-proof --profile rust --artifact "$tmp_rust_corpus/rust-corpus.json" --heldout-suite "$tmp_rust_corpus/rust-heldout.json" --focus-packet "$tmp_rust_corpus/rust-focus.json" --compile-evidence "$tmp_rust_corpus/rust-compile-evidence.json" --heldout-eval "$tmp_rust_corpus/rust-heldout-eval.json" --strict-density-evidence "$tmp_rust_corpus/strict-density.json" --multi-profile-density-evidence "$tmp_rust_corpus/multi-pass.json" --format json)"
jq -e '.verdict == "FINAL_PROOF_GATE_BLOCKED_BY_DENSITY_PROOF_DOCTOR" and .claim_boundary.nonlinear_memory_proven == false and .claim_boundary.llm_ready == false' <<<"$big_memory_final_proof_multi_profile_json" >/dev/null
jq -e '.final_proof_gate.general_nonlinear_memory_claim_ready == false and .final_proof_gate.density_proof_doctor_ready == false and .claim_boundary.blocked_by == ["density_proof_doctor_weak_or_missing"]' <<<"$big_memory_final_proof_multi_profile_json" >/dev/null
big_memory_final_proof_weak_doctor_json="$("$llmwave_big" memory-final-proof --profile rust --artifact "$tmp_rust_corpus/rust-corpus.json" --heldout-suite "$tmp_rust_corpus/rust-heldout.json" --focus-packet "$tmp_rust_corpus/rust-focus.json" --compile-evidence "$tmp_rust_corpus/rust-compile-evidence.json" --heldout-eval "$tmp_rust_corpus/rust-heldout-eval.json" --strict-density-evidence "$tmp_rust_corpus/strict-density.json" --multi-profile-density-evidence "$tmp_rust_corpus/multi-pass.json" --density-doctor-evidence "$tmp_rust_corpus/density-doctor.json" --format json)"
jq -e '.verdict == "FINAL_PROOF_GATE_BLOCKED_BY_DENSITY_PROOF_DOCTOR" and .big_corpus_gate.density_doctor_evidence_path != null and .claim_boundary.nonlinear_memory_proven == false' <<<"$big_memory_final_proof_weak_doctor_json" >/dev/null
jq -e '.final_proof_gate.density_proof_doctor_ready == false and .final_proof_gate.density_proof_doctor_strong == false and .claim_boundary.blocked_by == ["density_proof_doctor_weak_or_missing"]' <<<"$big_memory_final_proof_weak_doctor_json" >/dev/null
big_multi_profile_medium_json="$("$llmwave_big" multi-profile-density-suite --profile-evidence adversarial="$tmp_rust_corpus/adversarial-density.json" --profile-evidence contracts="$tmp_rust_corpus/contracts-density.json" --profile-evidence business="$tmp_rust_corpus/business-density.json" --out "$tmp_rust_corpus/multi-medium.json" --format json)"
jq -e '.verdict == "MULTI_PROFILE_NONLINEAR_MEMORY_PROVEN_NOT_LLM" and .suite.profile_count == 3 and .suite.passing_profile_count == 3' <<<"$big_multi_profile_medium_json" >/dev/null
big_density_doctor_medium_json="$("$llmwave_big" density-proof-doctor --suite "$tmp_rust_corpus/multi-medium.json" --min-fact-count 10 --out "$tmp_rust_corpus/density-doctor-medium.json" --format json)"
jq -e '.verdict == "DENSITY_PROOF_MEDIUM" and .gates.density_proof_doctor_medium_or_better == true and .gates.density_proof_doctor_strong == false and .gates.adversarial_profile_present == true' <<<"$big_density_doctor_medium_json" >/dev/null
big_density_ablation_json="$("$llmwave_big" density-ablation --suite "$tmp_rust_corpus/multi-medium.json" --out-hot-packet "$tmp_rust_corpus/density-ablation.hot" --format json)"
jq -e '.mode == "llmwave-big-density-ablation" and .verdict == "DENSITY_ABLATION_HAS_CRITICAL_PROFILES" and .baseline_duel.packed_beats_linear_baseline == true and ([.ablations[] | select(.impact == "CRITICAL")] | length) == 3 and .claim_boundary.proves_nonlinear_memory == false' <<<"$big_density_ablation_json" >/dev/null
jq -e '.unified_field.field_pass.version == "unified-field-pass-v1" and .unified_field.claim_boundary.not_nonlinear_memory_proof == true and .unified_field.runtime_contract.input_contract == "FieldPassInput"' <<<"$big_density_ablation_json" >/dev/null
jq -e '.runtime_path.runtime_path_kind == "density-suite-l2-l3-readonly-active-packet" and .runtime_path.active_packet_records == 3 and (.runtime_path.l3_proof_axes | index("anti_shortcut_rejection")) and .runtime_path.hot_loop_ready == false' <<<"$big_density_ablation_json" >/dev/null
jq -e '.hot_artifact.hot_packet_written == true and .hot_artifact.hot_packet_bytes == 64 and .hot_artifact.record_size_bytes == 16 and .hot_artifact.record_count == 3 and .hot_artifact.contains_json == false and .hot_artifact.hot_loop_ready == false' <<<"$big_density_ablation_json" >/dev/null
test -s "$tmp_rust_corpus/density-ablation.hot"
big_memory_final_proof_medium_doctor_json="$("$llmwave_big" memory-final-proof --profile rust --artifact "$tmp_rust_corpus/rust-corpus.json" --heldout-suite "$tmp_rust_corpus/rust-heldout.json" --focus-packet "$tmp_rust_corpus/rust-focus.json" --compile-evidence "$tmp_rust_corpus/rust-compile-evidence.json" --heldout-eval "$tmp_rust_corpus/rust-heldout-eval.json" --strict-density-evidence "$tmp_rust_corpus/strict-density.json" --multi-profile-density-evidence "$tmp_rust_corpus/multi-medium.json" --density-doctor-evidence "$tmp_rust_corpus/density-doctor-medium.json" --density-hot-packet "$tmp_rust_corpus/density-ablation.hot" --format json)"
jq -e '.verdict == "FINAL_PROOF_GATE_NONLINEAR_MEMORY_READY_NOT_LLM" and .final_proof_gate.general_nonlinear_memory_claim_ready == true and .claim_boundary.nonlinear_memory_proven == true and .claim_boundary.llm_ready == false' <<<"$big_memory_final_proof_medium_doctor_json" >/dev/null
jq -e '.claim_boundary.blocked_by == ["broad_llm_eval_missing"] and .final_proof_gate.density_proof_doctor_ready == true and .final_proof_gate.density_proof_doctor_strong == false and .final_proof_gate.density_hot_artifact_ready == true and .big_corpus_gate.density_hot_packet_path != null' <<<"$big_memory_final_proof_medium_doctor_json" >/dev/null
printf '%s\n' "$big_memory_final_proof_medium_doctor_json" >"$tmp_rust_corpus/memory-final-proof-medium.json"
cat >"$tmp_rust_corpus/broad-external.txt" <<'EOF_BROAD_EXTERNAL'
business|supplier_docs|Honglu|issues|invoice PI-03|external business evidence 1
business|supplier_docs|Rustrade|pays|Honglu|external business evidence 2
business|supplier_docs|invoice PI-03|supports|shipment docs|external business evidence 3
contracts|protocol_direction|supplier|authors|protocol|external contract evidence 1
contracts|protocol_direction|buyer|authors|original contract|external contract evidence 2
contracts|protocol_direction|protocol|changes|supplier obligation|external contract evidence 3
runtime|ime_runtime|IME not visible|requires|runtime engine check|external runtime evidence 1
runtime|ime_runtime|candidate scoring|is_not|IME activation|external runtime evidence 2
runtime|ime_runtime|runtime snapshot|shows|engine missing|external runtime evidence 3
customs|certification|Maria payment|covers|certification protocols|external customs evidence 1
customs|certification|customs declaration|requires|invoice evidence|external customs evidence 2
customs|certification|certification protocols|do_not_prove|customs payment|external customs evidence 3
code|owner_route|adapter|must_not_decide|core correction|external code evidence 1
code|owner_route|core owner|decides|correction|external code evidence 2
code|owner_route|ui adapter|displays|backend state|external code evidence 3
dialogue|dialogue_memory|reject shortcut|suppresses|next false route|external dialogue evidence 1
dialogue|dialogue_memory|correction|updates|session route|external dialogue evidence 2
dialogue|dialogue_memory|follow up|uses|accepted correction|external dialogue evidence 3
adversarial|adversarial_shortcut|same name|must_not_merge|different route|external adversarial evidence 1
adversarial|adversarial_shortcut|stale fact|must_not_override|current route|external adversarial evidence 2
adversarial|adversarial_shortcut|lexical match|must_not_force|role swap|external adversarial evidence 3
finance|payment_route|bank rate|does_not_prove|deposit rate|external finance evidence 1
finance|payment_route|payment route|must_match|payer beneficiary|external finance evidence 2
finance|payment_route|market shortcut|requires|source timestamp|external finance evidence 3
EOF_BROAD_EXTERNAL
big_broad_corpus_json="$("$llmwave_big" broad-corpus-build --source "$tmp_rust_corpus/broad-external.txt" --profile mixed-external-test --out "$tmp_rust_corpus/broad-corpus.json" --format json)"
jq -e '.mode == "llmwave-big-broad-corpus" and .fact_count == 24 and .route_count == 8 and .domain_count == 8 and .claim_boundary.external_corpus_loaded == true and .claim_boundary.llm_ready == false' <<<"$big_broad_corpus_json" >/dev/null
big_broad_doctor_json="$("$llmwave_big" broad-dataset-doctor --corpus "$tmp_rust_corpus/broad-corpus.json" --out "$tmp_rust_corpus/broad-dataset-doctor.json" --format json)"
jq -e '.mode == "llmwave-big-broad-dataset-doctor" and .verdict == "BROAD_DATASET_MEDIUM" and .quality.relation_count >= 8 and .quality.semantic_diversity_score >= 0.70 and .gates.semantic_diversity_ok == true and .gates.medium_or_better == true and .gates.strong == false and .claim_boundary.external_broad_corpus_ready == true' <<<"$big_broad_doctor_json" >/dev/null
big_broad_public_corpus_json="$("$llmwave_big" broad-corpus-build --source examples/llmwave-big-broad-public-corpus.txt --profile public-safe-strong-seed --out "$tmp_rust_corpus/broad-public-corpus.json" --format json)"
jq -e '.mode == "llmwave-big-broad-corpus" and .fact_count == 96 and .route_count == 32 and .domain_count == 8 and .claim_boundary.external_corpus_loaded == true and .claim_boundary.llm_ready == false' <<<"$big_broad_public_corpus_json" >/dev/null
big_broad_public_doctor_json="$("$llmwave_big" broad-dataset-doctor --corpus "$tmp_rust_corpus/broad-public-corpus.json" --out "$tmp_rust_corpus/broad-public-dataset-doctor.json" --format json)"
jq -e '.mode == "llmwave-big-broad-dataset-doctor" and .verdict == "BROAD_DATASET_STRONG" and .quality.fact_count == 96 and .quality.route_count == 32 and .quality.domain_count == 8 and .quality.route_balance == 1 and .quality.duplicate_pressure == 0 and .quality.semantic_diversity_score >= 0.70 and .gates.semantic_diversity_ok == true and .gates.strong == true and .claim_boundary.external_broad_corpus_ready == true and .claim_boundary.proves_general_llm == false' <<<"$big_broad_public_doctor_json" >/dev/null
big_broad_public_100k_corpus_json="$("$llmwave_big" broad-corpus-build --source examples/llmwave-big-broad-public-corpus-100k.txt --profile public-safe-100k --out "$tmp_rust_corpus/broad-public-100k-corpus.json" --format json)"
jq -e '.mode == "llmwave-big-broad-corpus" and .fact_count == 100000 and .route_count == 50 and .domain_count == 10 and .claim_boundary.external_corpus_loaded == true and .claim_boundary.llm_ready == false' <<<"$big_broad_public_100k_corpus_json" >/dev/null
big_broad_public_100k_doctor_json="$("$llmwave_big" broad-dataset-doctor --corpus "$tmp_rust_corpus/broad-public-100k-corpus.json" --out "$tmp_rust_corpus/broad-public-100k-dataset-doctor.json" --format json)"
jq -e '.mode == "llmwave-big-broad-dataset-doctor" and .verdict == "BROAD_DATASET_STRONG" and .quality.fact_count == 100000 and .quality.route_count == 50 and .quality.domain_count == 10 and .quality.route_balance == 1 and .quality.hub_dominance == 0.02 and .quality.duplicate_pressure == 0 and .quality.semantic_diversity_score >= 0.70 and .gates.semantic_diversity_ok == true and .gates.strong == true and .claim_boundary.external_broad_corpus_ready == true and .claim_boundary.proves_general_llm == false' <<<"$big_broad_public_100k_doctor_json" >/dev/null
big_broad_heldout_json="$("$llmwave_big" broad-heldout-build --corpus "$tmp_rust_corpus/broad-corpus.json" --out "$tmp_rust_corpus/broad-heldout.json" --max-cases 16 --format json)"
jq -e '.mode == "llmwave-big-broad-heldout-build" and .verdict == "BROAD_HELDOUT_SUITE_READY" and .metrics.generated_case_count == 16 and .metrics.withheld_fact_count == 16 and .metrics.covered_routes == 8 and .metrics.covered_domains == 8 and .metrics.family_count == 8 and .metrics.route_balance == 1 and .claim_boundary.heldout_suite_ready == true' <<<"$big_broad_heldout_json" >/dev/null
big_broad_focus_json="$("$llmwave_big" broad-focus-build --corpus "$tmp_rust_corpus/broad-corpus.json" --heldout-suite "$tmp_rust_corpus/broad-heldout.json" --out "$tmp_rust_corpus/broad-focus.json" --format json)"
jq -e '.mode == "llmwave-big-broad-focus-build" and .verdict == "BROAD_FOCUS_PACKET_READY" and .metrics.exact_withheld_facts_removed == 16 and .metrics.covered_domains_after == 8 and .metrics.covered_routes_after == 8 and .metrics.near_duplicate_leakage_count == 0 and .claim_boundary.route_balanced_focus_ready == true and .claim_boundary.hot_loop_ready == false' <<<"$big_broad_focus_json" >/dev/null
big_broad_eval_json="$("$llmwave_big" broad-eval-run --corpus "$tmp_rust_corpus/broad-corpus.json" --suite "$tmp_rust_corpus/broad-heldout.json" --focus-packet "$tmp_rust_corpus/broad-focus.json" --hot-packet "$tmp_rust_corpus/density-ablation.hot" --out "$tmp_rust_corpus/broad-eval.json" --format json)"
jq -e '.mode == "llmwave-big-broad-eval-run" and .verdict == "BROAD_EVAL_GENERATION_READY_NOT_CHAT" and .focus_summary.focus_loaded == true and .focus_summary.exact_withheld_facts_removed == 16 and .hot_packet.valid_density_packet == true and .claim_boundary.field_reasoning_ready == true and .claim_boundary.answer_generation_ready == true and .claim_boundary.llmwave_ready_candidate == false and .claim_boundary.llm_ready == false and (.claim_boundary.blocked_by | index("open_chat_loop_missing"))' <<<"$big_broad_eval_json" >/dev/null
big_broad_baseline_json="$("$llmwave_big" broad-baseline-duel --eval-report "$tmp_rust_corpus/broad-eval.json" --out "$tmp_rust_corpus/broad-baseline.json" --format json)"
jq -e '.mode == "llmwave-big-broad-baseline-duel" and .verdict == "BROAD_BASELINE_DUEL_TARGET_WIN" and .claim_boundary.broad_baseline_won == true and .claim_boundary.proves_general_llm == false' <<<"$big_broad_baseline_json" >/dev/null
big_broad_chat_loop_json="$("$llmwave_big" broad-chat-loop-eval --out "$tmp_rust_corpus/broad-chat-loop.json" --format json)"
jq -e '.mode == "llmwave-big-broad-chat-loop-eval" and .verdict == "BROAD_CHAT_LOOP_READY_NOT_GENERAL_LLM" and .claim_boundary.open_chat_loop_ready == true and .claim_boundary.full_llm_ready == false' <<<"$big_broad_chat_loop_json" >/dev/null
big_llmwave_readiness_json="$("$llmwave_big" llmwave-readiness --memory-final-proof "$tmp_rust_corpus/memory-final-proof-medium.json" --broad-dataset-doctor "$tmp_rust_corpus/broad-dataset-doctor.json" --broad-eval "$tmp_rust_corpus/broad-eval.json" --baseline-duel "$tmp_rust_corpus/broad-baseline.json" --chat-loop "$tmp_rust_corpus/broad-chat-loop.json" --out "$tmp_rust_corpus/llmwave-readiness.json" --format json)"
jq -e '.mode == "llmwave-big-readiness" and .verdict == "LLMWAVE_READY_CANDIDATE_EXTERNAL_MEDIUM" and .evidence.nonlinear_memory_proven == true and .evidence.density_hot_artifact_ready == true and .evidence.external_broad_corpus_ready == true and .evidence.external_strength == "external_medium" and .evidence.broad_baseline_won == true and .evidence.chat_loop_ready == true and .claim_boundary.field_reasoning_ready == true and .claim_boundary.answer_generation_ready == true and .claim_boundary.chat_loop_ready == true and .claim_boundary.external_broad_corpus_ready == true and .claim_boundary.llmwave_ready_candidate == true and .claim_boundary.llm_ready == false and (.claim_boundary.blocked_by | length) == 0' <<<"$big_llmwave_readiness_json" >/dev/null
rm -rf "$tmp_rust_corpus"
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
jq empty "$root/examples/llmwave-big-surface-corpus.json"
jq empty "$root/examples/llmwave-big-surface-corpus-ru.json"
jq empty "$root/examples/llmwave-big-raw-surface-corpus-ru.json"
jq empty "$root/examples/llmwave-big-raw-surface-corpus-ru-noisy.json"
jq empty "$root/examples/llmwave-big-raw-surface-corpus-ru-derived.json"
jq empty "$root/examples/llmwave-big-nonlinear-memory-corpus.json"
jq empty "$root/examples/eval-corpus.json"
jq empty "$root/examples/probe-corpus.json"
jq empty "$root/examples/waw-corpus.json"
jq empty "$root/examples/decode-corpus.json"
jq empty "$root/examples/pattern-learning-corpus.json"
jq empty "$root/examples/llmwave-corpus.json"
jq empty "$root/examples/token-lens-corpus.json"
jq empty "$root/examples/llmwave-memory-corpus.json"
jq empty "$root/examples/contract-gate.protocol-pass.json"
jq empty "$root/examples/contract-gate.protocol-watch.json"
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

contract_template_json="$("$root/target/debug/nanda" contract-gate --template --profile edo --format json)"
jq -e '.mode == "nanda-contract-gate-template" and .profile == "edo" and .packet.clauses[0].risk_tag == "unilateral_offset"' <<<"$contract_template_json" >/dev/null
jq -e '.packet.document_author and .packet.revision_source and .packet.clauses[0].effective_obligation_after_revision' <<<"$contract_template_json" >/dev/null
contract_pass_json="$("$root/target/debug/nanda" contract-gate --input "$root/examples/contract-gate.protocol-pass.json" --profile edo --format json)"
jq -e '.mode == "nanda-contract-gate" and .verdict == "STRUCTURAL_PASS_NOT_LEGAL_APPROVAL" and .safe_to_sign == false' <<<"$contract_pass_json" >/dev/null
jq -e '.role_first_pass.verdict == "PASS" and .protocol_direction_check.direction == "our_revision" and .multi_effect_clause_check.verdict == "PASS"' <<<"$contract_pass_json" >/dev/null
jq -e '(.risk_map.risk_tags | index("auto_acceptance")) and (.risk_map.risk_tags | index("unilateral_offset")) and (.negotiation_position.must_hold | length) >= 2 and .claim_boundary.structural_pass_is_not_legal_approval == true' <<<"$contract_pass_json" >/dev/null
tmp_contract_audit="$(mktemp -d)"
"$root/target/debug/nanda" contract-gate --input "$root/examples/contract-gate.protocol-pass.json" --profile edo --audit-dir "$tmp_contract_audit" --format json >/dev/null
test -s "$tmp_contract_audit/contract-gate-report.json"
test -s "$tmp_contract_audit/negotiation-position.json"
rm -rf "$tmp_contract_audit"
set +e
contract_watch_json="$("$root/target/debug/nanda" contract-gate --input "$root/examples/contract-gate.protocol-watch.json" --profile protocol --format json)"
contract_watch_status=$?
set -e
test "$contract_watch_status" -eq 3
jq -e '.verdict == "WATCH" and (.watch_reasons | index("protocol_direction_unknown_or_unmatched")) and (.watch_reasons | index("same_clause_multi_effect_requires_subclaim_split")) and (.watch_reasons | index("risk_tags_missing"))' <<<"$contract_watch_json" >/dev/null
jq -e '.multi_effect_clause_check.repair == "split_by_subclaim" and ([.split_plan[].route] | index("same-clause-subclaims"))' <<<"$contract_watch_json" >/dev/null

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
jq -e '.hierarchical_decision.agent_interpretation == "STRUCTURALLY_ACCEPTED_WITH_SPLIT"' <<<"$hgate_json" >/dev/null
jq -e '.hierarchical_decision.accepted_with_split == true' <<<"$hgate_json" >/dev/null
jq -e '.hierarchical_decision.global_verdict == "WATCH"' <<<"$hgate_json" >/dev/null
jq -e '.hierarchical_decision.global_size_only == true' <<<"$hgate_json" >/dev/null
jq -e '.hierarchical_decision.local_pass == 17 and .hierarchical_decision.branches == 17' <<<"$hgate_json" >/dev/null
grep -q "STRUCTURALLY_ACCEPTED_WITH_SPLIT" "$root/nanda-structural-gate/references/hierarchical-gate.md"
grep -q "claim_boundary_gate" "$root/nanda-structural-gate/references/hierarchical-gate.md"
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
jq -e '.unified_field.family == "structural" and .unified_field.compute_probe.version == "unified-field-compute-v1" and (.unified_field.query.signature | type == "string" and length == 16)' <<<"$search_json" >/dev/null
jq -e '.unified_field.field_pass.version == "unified-field-pass-v1" and .unified_field.field_pass.family == "structural" and .unified_field.field_pass.safe_to_answer == false' <<<"$search_json" >/dev/null
jq -e '.unified_field.runtime_contract.version == "unified-field-runtime-v1" and .unified_field.runtime_contract.input_contract == "FieldPassInput" and .unified_field.runtime_contract.output_contract == "FieldPassReport" and .unified_field.runtime_contract.field_core_as_sole_engine == false' <<<"$search_json" >/dev/null
jq -e '.field_runtime.version == "unified-field-runtime-v1" and .field_runtime.mode == "structural-dual-run" and .field_runtime.peak_matches == true and .field_runtime.field_not_more_permissive == true' <<<"$search_json" >/dev/null
jq -e '.field_engine.version == "structural-field-engine-v1" and .field_engine.mode == "cutover" and .field_engine.field_participates == true and .field_engine.selected_engine == "field-core-cutover" and .field_engine.cutover_applied == true and .field_engine.field_core_as_sole_engine == true and .field_engine.field_core_as_structural_sole_engine == true' <<<"$search_json" >/dev/null
legacy_search_json="$("$search" "$root/examples/triad-packet.interference-search.json" --input-format json --top-k 3 --field-engine legacy)"
jq -e '.field_engine.version == "structural-field-engine-v1" and .field_engine.mode == "legacy" and .field_engine.field_participates == false and .field_engine.selected_engine == "structural-domain" and .field_engine.top_level_behavior_changed == false and .field_engine.field_core_as_sole_engine == false' <<<"$legacy_search_json" >/dev/null
jq -e '.unified_field.memory_delta.replayable_into_next_pass | type == "boolean"' <<<"$search_json" >/dev/null
jq -e '.unified_field.record.records == 10 and .unified_field.record.routes >= 1 and .unified_field.record.groups >= 1 and .unified_field.peak.support_count == 4 and .unified_field.peak.anti_support_count == 5' <<<"$search_json" >/dev/null

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
set +e
cache_proof_json="$("$proof" "$root/examples/triad-packet.route-balanced-focus.json" --input-format json --query "lower operator debt route" --fast --cache-dir "$tmp_cache")"
cache_proof_status=$?
set -e
if [[ "$cache_proof_status" -ne 0 && "$cache_proof_status" -ne 3 ]]; then
  echo "expected fast cache proof to return PASS or WATCH status" >&2
  echo "$cache_proof_json" >&2
  exit 1
fi
jq -e '.proof_mode == "fast-focused" and .focus_cache.state == "CACHE_HIT" and (.reason_codes | index("RAW_SEARCH_SKIPPED"))' <<<"$cache_proof_json" >/dev/null
cache_manifest="$(find "$tmp_cache" -type f -name '*.manifest.json' | head -n 1)"
set +e
cache_only_json="$("$proof" --cache-only "$cache_manifest")"
cache_only_status=$?
set -e
if [[ "$cache_only_status" -ne 0 && "$cache_only_status" -ne 3 ]]; then
  echo "expected cache-only proof to return PASS or WATCH status" >&2
  echo "$cache_only_json" >&2
  exit 1
fi
jq -e '.proof_mode == "cache-only-focused" and .focus_cache.state == "CACHE_ONLY_HIT" and .corpus.state == "CORPUS_NOT_LOADED" and (.reason_codes | index("CORPUS_NOT_LOADED"))' <<<"$cache_only_json" >/dev/null
serve_cache_json="$(printf '{"command":"proof_cache_only","manifest":"%s"}\n{"command":"proof_cache_only","manifest":"%s"}\n' "$cache_manifest" "$cache_manifest" | "$serve")"
jq -e 'length == 2 and .[0].ok == true and .[0].elapsed_ms >= 0 and .[0].result.proof_mode == "cache-only-focused" and .[0].result.focus_cache.state == "CACHE_ONLY_HIT" and .[0].result.serve_cache.state == "SERVE_MEMORY_WARMED" and .[1].ok == true and .[1].result.serve_cache.state == "SERVE_PROOF_HIT"' <<<"$(jq -s . <<<"$serve_cache_json")" >/dev/null
serve_compact_json="$(printf '{"command":"proof_cache_only","manifest":"%s","response":"compact"}\n{"command":"proof_cache_only","manifest":"%s","response":"compact"}\n' "$cache_manifest" "$cache_manifest" | "$serve")"
jq -e 'length == 2 and .[0].ok == true and .[0].result.mode == "proof-cache-only-compact" and ((.[0].result.proof_state | length) > 0) and .[0].result.focused_search == null and .[1].result.serve_cache.state == "SERVE_PROOF_HIT"' <<<"$(jq -s . <<<"$serve_compact_json")" >/dev/null
cat >"$tmp_cache/field-report.json" <<'EOF_FIELD_SERVE'
{
  "peak_decision": {"state": "PASS", "safe_to_answer": true},
  "field_state_machine": {"state": "FIELD_FOCUSED", "action": "answer"},
  "query": {"text": "serve field report"},
  "peaks": [{"peak": "runtime", "score": 0.8}],
  "supporting_triads": [{"id": "t1"}]
}
EOF_FIELD_SERVE
serve_field_report_json="$(printf '{"command":"field_report","input":"%s"}\n{"command":"field_report","input":"%s"}\n' "$tmp_cache/field-report.json" "$tmp_cache/field-report.json" | "$serve")"
jq -e 'length == 2 and .[0].ok == true and .[0].result.family == "structural" and .[0].result.compute_probe.version == "unified-field-compute-v1" and .[0].result.serve_cache.state == "SERVE_FIELD_REPORT_WARMED" and .[1].ok == true and .[1].result.serve_cache.state == "SERVE_FIELD_REPORT_HIT"' <<<"$(jq -s . <<<"$serve_field_report_json")" >/dev/null
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
jq -e '.field_runtime.mode == "structural-dual-run" and .field_runtime.peak_matches == true and .field_runtime.state_family_matches == true and .field_runtime.field_not_more_permissive == true and .field_runtime.cutover_ready == true' <<<"$noisy_search_json" >/dev/null
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
jq -e '.unified_field.family == "packed" and .unified_field.compute_probe.version == "unified-field-compute-v1" and (.unified_field.query.signature | type == "string" and length == 16)' <<<"$pack6m_json" >/dev/null
jq -e '.unified_field.field_pass.version == "unified-field-pass-v1" and .unified_field.field_pass.family == "packed" and .unified_field.field_pass.safe_to_answer == false' <<<"$pack6m_json" >/dev/null
jq -e '.field_runtime.version == "unified-field-runtime-v1" and .field_runtime.mode == "packed-dual-run" and .field_runtime.peak_matches == true and .field_runtime.state_family_matches == true and .field_runtime.field_not_more_permissive == true and .field_runtime.cutover_ready == true' <<<"$pack6m_json" >/dev/null
jq -e '.packed_field_engine.version == "packed-field-engine-guard-v1" and .packed_field_engine.mode == "legacy" and .packed_field_engine.selected_engine == "packed-hot-core" and .packed_field_engine.field_participates == false and .packed_field_engine.field_core_as_sole_engine == false and .packed_field_engine.field_core_as_packed_hot_engine == false and .packed_field_engine.top_level_behavior_changed == false' <<<"$pack6m_json" >/dev/null
jq -e '.field_record_view.version == "packed-field-record-view-v1" and .field_record_view.source_record == "PackedTriad32" and .field_record_view.zero_copy == true and .field_record_view.hot_loop_safe == true and (.field_record_view.inner_loop_forbidden | index("json")) and (.field_record_view.inner_loop_forbidden | index("heap"))' <<<"$pack6m_json" >/dev/null
jq -e '.unified_field.record.records == 10 and .unified_field.record.routes == 3 and .unified_field.record.groups == 3 and .unified_field.peak.target == "route:2" and .unified_field.peak.support_count == 1 and .unified_field.peak.anti_support_count == 2 and .unified_field.anti_wave.suppression_energy == 256' <<<"$pack6m_json" >/dev/null
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
pack6m_candidate_json="$("$pack6m" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --field-engine candidate)"
jq -e '.packed_field_engine.mode == "candidate" and .packed_field_engine.field_participates == true and .packed_field_engine.candidate_allowed == true and .packed_field_engine.field_core_as_packed_engine_candidate == true and .packed_field_engine.selected_engine == "field-core-packed-candidate" and .packed_field_engine.top_level_behavior_changed == false and .packed_field_engine.field_core_as_sole_engine == false' <<<"$pack6m_candidate_json" >/dev/null
pack6m_cutover_json="$("$pack6m" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --field-engine cutover)"
jq -e '.packed_field_engine.mode == "cutover" and .packed_field_engine.cutover_requested == true and .packed_field_engine.candidate_allowed == true and .packed_field_engine.cutover_applied == true and .packed_field_engine.selected_engine == "field-core-packed-cutover" and .packed_field_engine.field_core_as_packed_hot_engine == true and .packed_field_engine.field_core_as_packed_sole_engine == true and .packed_field_engine.hot_core_guard.packed_hot_core_exception == false and .packed_field_engine.hot_core_guard.satisfied_by_typed_packed_decision == true and (.packed_field_engine.cutover_blocked_reason | length == 0) and .packed_field_engine.top_level_behavior_changed == true and .packed_field_cutover.applied == true' <<<"$pack6m_cutover_json" >/dev/null
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
jq -e '.verdict == "WATCH" and .field_state == "FIELD_FOCUSED" and .safe_to_answer == false and .top_peak == "certification" and .field_cutover.applied == true and .field_engine.field_core_as_sole_engine == true' <<<"$trap_search_json" >/dev/null
jq -e '.field_runtime.mode == "structural-dual-run" and .field_runtime.cutover_ready == true and .field_runtime.field_safe_to_answer == false' <<<"$trap_search_json" >/dev/null
trap_legacy_search_json="$("$search" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --top-k 3 --field-engine legacy)"
jq -e '.verdict == "PASS" and .field_state == "FIELD_FOCUSED" and .safe_to_answer == true and .top_peak == "certification" and .field_engine.mode == "legacy"' <<<"$trap_legacy_search_json" >/dev/null
trap_shadow_search_json="$("$search" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --top-k 3 --field-engine shadow)"
jq -e '.top_peak == "certification" and .field_engine.mode == "shadow" and .field_engine.field_participates == true and .field_engine.selected_engine == "structural-domain" and .field_engine.candidate_allowed == false and .field_engine.top_level_behavior_changed == false' <<<"$trap_shadow_search_json" >/dev/null
trap_candidate_search_json="$("$search" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --top-k 3 --field-engine candidate)"
jq -e '.top_peak == "certification" and .field_engine.mode == "candidate" and .field_engine.field_participates == true and .field_engine.selected_engine == "field-core-candidate" and .field_engine.selected.peak == "certification" and .field_engine.candidate_allowed == true and .field_engine.field_core_as_sole_engine == false and .field_engine.top_level_behavior_changed == false' <<<"$trap_candidate_search_json" >/dev/null
trap_cutover_search_json="$("$search" "$root/examples/triad-packet.interference-search-route-trap.json" --input-format json --top-k 3 --field-engine cutover)"
jq -e '.top_peak == "certification" and .verdict == "WATCH" and .safe_to_answer == false and .field_cutover.applied == true and .field_cutover.old_verdict == "PASS" and .field_cutover.new_verdict == "WATCH" and .field_engine.mode == "cutover" and .field_engine.selected_engine == "field-core-cutover" and .field_engine.cutover_applied == true and .field_engine.field_core_as_sole_engine == true and .field_engine.field_core_as_structural_sole_engine == true and .field_engine.top_level_behavior_changed == true' <<<"$trap_cutover_search_json" >/dev/null
noisy_cutover_search_json="$("$search" "$root/examples/triad-packet.interference-search-noisy.json" --input-format json --top-k 3 --field-engine cutover)"
jq -e '.field_cutover.applied == true and .field_cutover.old_verdict == "WATCH" and .field_cutover.new_verdict == "VETO" and .verdict == "VETO" and .field_state == "FIELD_CONTESTED" and .safe_to_answer == false and .field_engine.cutover_applied == true and .field_engine.field_core_as_sole_engine == true and .field_engine.field_core_as_structural_sole_engine == true' <<<"$noisy_cutover_search_json" >/dev/null
reversed_cutover_search_json="$("$search" "$root/examples/triad-packet.polarization-reversed-stop.json" --input-format json --top-k 3 --field-engine cutover)"
jq -e '.field_cutover.applied == true and .safe_to_answer == false and .field_state == "FIELD_REVERSED" and .field_engine.cutover_applied == true' <<<"$reversed_cutover_search_json" >/dev/null
thin_cutover_search_json="$("$search" "$root/examples/triad-packet.negative-shortcut-lanes.json" --input-format json --top-k 3 --field-engine cutover)"
jq -e '.field_cutover.applied == true and .verdict == "WATCH" and .safe_to_answer == false and .field_state == "FIELD_THIN" and .field_engine.cutover_applied == true' <<<"$thin_cutover_search_json" >/dev/null
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
jq -e '.refactor_plan.mode == "repo-code-map"' <<<"$dogfood_refactor_json" >/dev/null
jq -e '.refactor_plan.clusters | length > 0' <<<"$dogfood_refactor_json" >/dev/null
jq -e '.refactor_plan.risk_files | length > 0' <<<"$dogfood_refactor_json" >/dev/null
code_map_json="$("$code_mapper" "$root/src/main.rs" --format json)"
jq -e '.mode == "code-map"' <<<"$code_map_json" >/dev/null
jq -e '.clusters | length > 0' <<<"$code_map_json" >/dev/null
jq -e '.clusters[] | select(.cluster == "cli-router")' <<<"$code_map_json" >/dev/null
code_map_repo_json="$("$code_mapper" "$root" --format json)"
jq -e '.mode == "repo-code-map" and (.input | length) > 0 and .total_files > 0 and (.routes | length) > 0 and (.risk_files | length) > 0' <<<"$code_map_repo_json" >/dev/null
tmp_kernel_repo="$(mktemp -d)"
mkdir -p "$tmp_kernel_repo/fs" "$tmp_kernel_repo/kernel/bpf" "$tmp_kernel_repo/io_uring" "$tmp_kernel_repo/include/uapi/linux"
cat >"$tmp_kernel_repo/fs/namei.c" <<'EOF_KERNEL_NAMEI'
static int nd_jump_root(struct nameidata *nd)
{
    if (nd->flags & LOOKUP_BENEATH)
        return -EXDEV;
    return 0;
}

static bool path_connected(struct vfsmount *mnt, struct dentry *dentry)
{
    return is_subdir(dentry, mnt->mnt_root);
}
EOF_KERNEL_NAMEI
cat >"$tmp_kernel_repo/kernel/bpf/verifier.c" <<'EOF_KERNEL_BPF'
static int sanitize_speculative_path(struct bpf_verifier_env *env)
{
    return push_stack(env, 1, 0, true);
}

static int sanitize_ptr_alu(struct bpf_verifier_env *env)
{
    return sanitize_speculative_path(env);
}
EOF_KERNEL_BPF
cat >"$tmp_kernel_repo/io_uring/register.c" <<'EOF_KERNEL_URING'
static int io_register_restrictions(struct io_ring_ctx *ctx)
{
    if (!(ctx->flags & IORING_SETUP_R_DISABLED))
        return -EBADFD;
    return 0;
}

static inline bool io_check_restriction(struct io_ring_ctx *ctx)
{
    return test_bit(ctx->opcode, ctx->restrictions.sqe_op);
}
EOF_KERNEL_URING
cat >"$tmp_kernel_repo/include/uapi/linux/openat2.h" <<'EOF_KERNEL_OPENAT2'
#define RESOLVE_BENEATH 0x08
#define RESOLVE_IN_ROOT 0x10
EOF_KERNEL_OPENAT2
kernel_code_map_json="$("$code_mapper" "$tmp_kernel_repo" --format json)"
jq -e '.mode == "repo-code-map" and .total_files == 4 and .selected_files == 4' <<<"$kernel_code_map_json" >/dev/null
jq -e '.routes | index("kernel-vfs-path-flow") and index("kernel-bpf-verifier-flow") and index("kernel-io-uring-flow")' <<<"$kernel_code_map_json" >/dev/null
jq -e '.clusters[] | select(.input_file == "fs/namei.c" and .cluster == "kernel-vfs-path" and .risk == "HIGH" and (.symbol_kinds | index("function")))' <<<"$kernel_code_map_json" >/dev/null
jq -e '.clusters[] | select(.input_file == "kernel/bpf/verifier.c" and .cluster == "kernel-bpf-speculation" and .risk == "HIGH")' <<<"$kernel_code_map_json" >/dev/null
jq -e '.clusters[] | select(.input_file == "io_uring/register.c" and .cluster == "kernel-io-uring-restrictions" and .risk == "HIGH")' <<<"$kernel_code_map_json" >/dev/null
jq -e '.clusters[] | select(.input_file == "include/uapi/linux/openat2.h" and .cluster == "kernel-vfs-path" and (.symbol_kinds | index("macro")))' <<<"$kernel_code_map_json" >/dev/null
rm -rf "$tmp_kernel_repo"
route_field_json="$("$mapper" "$root/examples/triad-packet.route-field-owner-conflict.json" --input-format json --format json)"
jq -e '.route_field.routes.correction.owners | index("core-correction-owner")' <<<"$route_field_json" >/dev/null
jq -e '.owner_gravity.conflicts[] | select(.kind == "duplicate_decision_owner" and .route == "correction")' <<<"$route_field_json" >/dev/null
jq -e '.negative_routes.hits[] | select(.rule == "adapter_or_ui_must_not_decide" and .triad == "c1")' <<<"$route_field_json" >/dev/null
jq -e '.structural_energy.owner_conflict == 1 and .structural_energy.adapter_leak_risk == 1 and (.repair_queue | length) >= 2' <<<"$route_field_json" >/dev/null
set +e
route_field_dogfood="$("$dogfood" "$root/examples/triad-packet.route-field-owner-conflict.json" --format json)"
route_field_dogfood_status=$?
set -e
if [[ "$route_field_dogfood_status" -ne 1 ]]; then
  echo "expected route-field owner conflict dogfood to VETO" >&2
  echo "$route_field_dogfood" >&2
  exit 1
fi
jq -e '.agent_decision.action == "REPAIR_REQUIRED" and .agent_decision.owner_conflict == 1 and .agent_decision.negative_route_hits == 1' <<<"$route_field_dogfood" >/dev/null
failure_runtime_json="$("$mapper" "$root/examples/triad-packet.codex-failure-runtime-mismatch.json" --input-format json --format json)"
jq -e '.codex_failure_field.verdict == "VETO" and (.codex_failure_field.reason_codes | index("symptom_action_mismatch")) and (.codex_failure_field.reason_codes | index("runtime_blindness"))' <<<"$failure_runtime_json" >/dev/null
set +e
failure_runtime_gate_json="$("$checker" --triads "$root/examples/triad-packet.codex-failure-runtime-mismatch.json" --format json)"
failure_runtime_gate_status=$?
set -e
if [[ "$failure_runtime_gate_status" -ne 1 ]]; then
  echo "expected failure runtime gate to VETO" >&2
  echo "$failure_runtime_gate_json" >&2
  exit 1
fi
jq -e '.verdict == "VETO" and .agent_decision.action == "REPAIR_REQUIRED" and .codex_failure_field.verdict == "VETO" and (.agent_decision.codex_failure_reasons | index("symptom_action_mismatch"))' <<<"$failure_runtime_gate_json" >/dev/null
jq -e '.repair_queue[0].kind == "symptom_action_mismatch" and (.repair_queue[0].repair | contains("runtime route first"))' <<<"$failure_runtime_gate_json" >/dev/null
set +e
failure_stop_json="$("$dogfood" "$root/examples/triad-packet.codex-failure-hard-stop.json" --format json)"
failure_stop_status=$?
set -e
if [[ "$failure_stop_status" -ne 1 ]]; then
  echo "expected hard stop dogfood to VETO exit" >&2
  echo "$failure_stop_json" >&2
  exit 1
fi
jq -e '.agent_decision.action == "HARD_STOP" and .agent_decision.codex_failure_verdict == "HARD_STOP" and .agent_decision.safe_to_edit == false' <<<"$failure_stop_json" >/dev/null
failure_namespace_json="$("$mapper" "$root/examples/triad-packet.codex-failure-namespace-watch.json" --input-format json --format json)"
jq -e '.codex_failure_field.verdict == "ANALYSIS_INSUFFICIENT" and (.codex_failure_field.reason_codes | index("namespace_confusion"))' <<<"$failure_namespace_json" >/dev/null
tmp_auto_repo="$(mktemp -d)"
mkdir -p "$tmp_auto_repo/src" "$tmp_auto_repo/scripts"
printf 'fn main() {}' >"$tmp_auto_repo/src/main.rs"
printf 'echo install' >"$tmp_auto_repo/scripts/install.sh"
set +e
auto_dogfood_json="$("$dogfood" "$tmp_auto_repo" --format json)"
auto_dogfood_status=$?
set -e
if [[ "$auto_dogfood_status" -ne 3 ]]; then
  echo "expected auto dogfood to return WATCH/REVIEW" >&2
  echo "$auto_dogfood_json" >&2
  exit 1
fi
jq -e '.agent_decision.action == "REVIEW_REQUIRED" and .agent_decision.codex_failure_verdict == "ANALYSIS_INSUFFICIENT" and (.input | contains("repo-auto-dogfood"))' <<<"$auto_dogfood_json" >/dev/null
set +e
auto_dogfood_refactor_json="$("$dogfood" "$tmp_auto_repo" --refactor-plan --format json)"
auto_dogfood_refactor_status=$?
set -e
test "$auto_dogfood_refactor_status" -eq 3
jq -e '.refactor_plan.mode == "repo-code-map" and (.refactor_plan.files | length) >= 1 and (.refactor_plan.risk_files | length) >= 0' <<<"$auto_dogfood_refactor_json" >/dev/null
rm -rf "$tmp_auto_repo"
tmp_owner_repo="$(mktemp -d)"
mkdir -p "$tmp_owner_repo/src/bin/lay_daemon" "$tmp_owner_repo/src/runtime"
printf 'fn main() {}' >"$tmp_owner_repo/src/bin/lay_daemon.rs"
printf 'fn ime_candidate_event() {}' >"$tmp_owner_repo/src/bin/lay_daemon/ime_candidate.rs"
printf 'fn handle_manual_trigger_runtime() {}' >"$tmp_owner_repo/src/runtime/manual_trigger_runtime.rs"
set +e
owner_auto_json="$("$dogfood" "$tmp_owner_repo" --format json)"
owner_auto_status=$?
set -e
if [[ "$owner_auto_status" -ne 3 ]]; then
  echo "expected owner auto dogfood to return WATCH/REVIEW" >&2
  echo "$owner_auto_json" >&2
  exit 1
fi
jq -e '.comb_tree.map.route_field.routes["runtime-flow"].owners | index("src::bin::lay_daemon")' <<<"$owner_auto_json" >/dev/null
jq -e '(.comb_tree.map.route_field.routes["runtime-flow"].owners | index("src::bin::lay_daemon.rs")) == null' <<<"$owner_auto_json" >/dev/null
jq -e '.comb_tree.map.route_field.routes["runtime-flow"].evidence_paths | index("src/bin/lay_daemon/ime_candidate.rs")' <<<"$owner_auto_json" >/dev/null
jq -e '.comb_tree.map.route_field.routes["manual-trigger-flow"].evidence_paths | index("src/runtime/manual_trigger_runtime.rs")' <<<"$owner_auto_json" >/dev/null
rm -rf "$tmp_owner_repo"
tmp_state_repo="$(mktemp -d)"
mkdir -p "$tmp_state_repo/src"
cat >"$tmp_state_repo/src/event.rs" <<'EOF_EVENT'
fn handle_double_shift_event() {
    transition_manual_trigger_state();
}

fn transition_manual_trigger_state() {}
EOF_EVENT
state_code_json="$("$code_mapper" "$tmp_state_repo/src/event.rs" --format json --min-cluster-functions 1)"
jq -e '.clusters[] | select(.cluster == "manual-trigger" and .risk == "HIGH" and (.risk_reason | contains("ROUTE_CRITICAL")))' <<<"$state_code_json" >/dev/null
rm -rf "$tmp_state_repo"
tmp_atlas_repo="$(mktemp -d)"
mkdir -p "$tmp_atlas_repo/src/bin/lay_daemon" "$tmp_atlas_repo/src/bin/lay_ibus_engine" "$tmp_atlas_repo/src/runtime"
mkdir -p "$tmp_atlas_repo/extension/gnome"
printf 'fn main() {}' >"$tmp_atlas_repo/src/bin/lay_daemon.rs"
printf 'fn main() {}' >"$tmp_atlas_repo/src/bin/lay_ibus_engine.rs"
printf 'fn handle_manual_trigger_runtime() {}' >"$tmp_atlas_repo/src/runtime/manual_trigger_runtime.rs"
printf 'pub fn toggle_manual_mode() {}' >"$tmp_atlas_repo/src/manual_toggle.rs"
cat >"$tmp_atlas_repo/Cargo.toml" <<'EOF_ATLAS_VERSION'
[package]
name = "generic-app"
version = "1.2.3"

[dependencies]
gtk = "0.3.2"
glib = "5.15.0"
EOF_ATLAS_VERSION
cat >"$tmp_atlas_repo/Cargo.lock" <<'EOF_ATLAS_VERSION'
[[package]]
name = "generic-app"
version = "1.2.3"
EOF_ATLAS_VERSION
cat >"$tmp_atlas_repo/VERSIONING.md" <<'EOF_ATLAS_VERSION'
Current version: 1.2.3
EOF_ATLAS_VERSION
cat >"$tmp_atlas_repo/extension/gnome/metadata.json" <<'EOF_ATLAS_VERSION'
{"version-name":"1.2.3","version":3}
EOF_ATLAS_VERSION
cat >"$tmp_atlas_repo/extension/gnome/prefs.js" <<'EOF_ATLAS_VERSION'
const APP_VERSION = '1.2.3';
EOF_ATLAS_VERSION
cat >"$tmp_atlas_repo/extension/gnome/settings.js" <<'EOF_ATLAS_VERSION'
export const APP_VERSION = '1.2.3';
const label = new Gtk.Label({label: `Lay ${APP_VERSION}`});
EOF_ATLAS_VERSION
cat >"$tmp_atlas_repo/extension/gnome/tray_support.js" <<'EOF_ATLAS_VERSION'
const APP_VERSION = '1.2.3';
EOF_ATLAS_VERSION
atlas_path="$tmp_atlas_repo/.nanda/route-atlas.json"
atlas_json="$("$build_atlas" "$tmp_atlas_repo" --out "$atlas_path" --format json)"
jq -e '.mode == "route-atlas" and (.input | length) > 0 and (.output | length) > 0 and .routes["runtime-flow"] and .routes["ime-display-flow"] and .routes["manual-trigger-flow"] and .shared_contracts["shared.manual_toggle_contract"] and .shared_contracts["shared.version_bump_contract"] and .written_to' <<<"$atlas_json" >/dev/null
test -s "$atlas_path"
atlas_file_json="$(cat "$atlas_path")"
jq -e '(.input | length) > 0 and (.output | length) > 0' <<<"$atlas_file_json" >/dev/null
dogfood_atlas_json="$("$dogfood" "$tmp_atlas_repo" --build-atlas --atlas-out "$tmp_atlas_repo/.nanda/dogfood-atlas.json" --format json)"
jq -e '.mode == "route-atlas" and .routes["ime-display-flow"]' <<<"$dogfood_atlas_json" >/dev/null
guard_pass_json="$("$guard_action" "$atlas_path" --symptom "IME not visible" --action-id "ime.activate_engine" --format json)"
jq -e '.verdict == "PASS" and .safe_to_edit == true and .route == "ime-display-flow"' <<<"$guard_pass_json" >/dev/null
printf '{"ibus_engine":"xkb:ru::rus","processes":["lay-daemon"],"config":{"text_backend":"ime"}}\n' >"$tmp_atlas_repo/runtime-snapshot.json"
set +e
guard_veto_json="$("$guard_action" "$atlas_path" --symptom "IME not visible" --action-id "nanda.edit_candidate_generation" --runtime-snapshot "$tmp_atlas_repo/runtime-snapshot.json" --format json)"
guard_veto_status=$?
set -e
test "$guard_veto_status" -eq 1
jq -e '.verdict == "VETO" and (.reason_codes | index("symptom_action_mismatch")) and .safe_to_edit == false' <<<"$guard_veto_json" >/dev/null
set +e
guard_stop_json="$("$guard_action" "$atlas_path" --symptom "СТОЙ не трогай код" --action-id "ime.activate_engine" --format json)"
guard_stop_status=$?
set -e
test "$guard_stop_status" -eq 1
jq -e '.verdict == "HARD_STOP" and .safe_to_edit == false' <<<"$guard_stop_json" >/dev/null
cat >"$tmp_atlas_repo/ime.diff" <<'EOF_DIFF'
diff --git a/src/bin/lay_ibus_engine.rs b/src/bin/lay_ibus_engine.rs
--- a/src/bin/lay_ibus_engine.rs
+++ b/src/bin/lay_ibus_engine.rs
@@ -1 +1 @@
-fn main() {}
+fn main() { }
EOF_DIFF
guard_diff_pass="$("$guard_diff" "$atlas_path" --action-id "ime.show_candidate" --diff "$tmp_atlas_repo/ime.diff" --format json)"
jq -e '.verdict == "PASS" and .safe_to_edit == true' <<<"$guard_diff_pass" >/dev/null
jq -e '.boundary_diff_kernel.owner == "field_core::boundary::diff" and .boundary_diff_kernel.diff_verdict == "DIFF_KEEP" and .boundary_diff_kernel.field_equivalence.field_not_more_permissive == true' <<<"$guard_diff_pass" >/dev/null
touch "$tmp_atlas_repo/empty.diff"
set +e
guard_diff_empty="$("$guard_diff" "$atlas_path" --action-id "ime.show_candidate" --diff "$tmp_atlas_repo/empty.diff" --format json)"
guard_diff_empty_status=$?
set -e
test "$guard_diff_empty_status" -eq 3
jq -e '.verdict == "WATCH" and .safe_to_edit == false and .reason == "empty_or_unreadable_diff"' <<<"$guard_diff_empty" >/dev/null
cat >"$tmp_atlas_repo/mixed.diff" <<'EOF_DIFF'
diff --git a/src/bin/lay_ibus_engine.rs b/src/bin/lay_ibus_engine.rs
--- a/src/bin/lay_ibus_engine.rs
+++ b/src/bin/lay_ibus_engine.rs
diff --git a/src/runtime/manual_trigger_runtime.rs b/src/runtime/manual_trigger_runtime.rs
--- a/src/runtime/manual_trigger_runtime.rs
+++ b/src/runtime/manual_trigger_runtime.rs
EOF_DIFF
set +e
guard_diff_veto="$("$guard_diff" "$atlas_path" --action-id "ime.show_candidate" --diff "$tmp_atlas_repo/mixed.diff" --format json)"
guard_diff_status=$?
set -e
test "$guard_diff_status" -eq 1
jq -e '.verdict == "VETO" and (.foreign_routes | index("manual-trigger-flow")) and .route_crossing_report.decision == "allowed only if action_id is an explicit shared contract for these routes"' <<<"$guard_diff_veto" >/dev/null
cat >"$tmp_atlas_repo/shared.diff" <<'EOF_DIFF'
diff --git a/src/manual_toggle.rs b/src/manual_toggle.rs
--- a/src/manual_toggle.rs
+++ b/src/manual_toggle.rs
diff --git a/src/bin/lay_ibus_engine.rs b/src/bin/lay_ibus_engine.rs
--- a/src/bin/lay_ibus_engine.rs
+++ b/src/bin/lay_ibus_engine.rs
EOF_DIFF
guard_diff_shared="$("$guard_diff" "$atlas_path" --action-id "shared.manual_toggle_contract" --diff "$tmp_atlas_repo/shared.diff" --format json)"
jq -e '.verdict == "PASS" and .safe_to_edit == true and .reason == "shared_contract_allows_route_crossing" and (.changed_routes | index("source-flow")) and (.changed_routes | index("ime-display-flow")) and (.shared_candidates | index("src/manual_toggle.rs")) and .route_crossing_report.decision == "allowed by shared.manual_toggle_contract"' <<<"$guard_diff_shared" >/dev/null
set +e
guard_diff_shared_veto="$("$guard_diff" "$atlas_path" --action-id "ime.show_candidate" --diff "$tmp_atlas_repo/shared.diff" --format json)"
guard_diff_shared_veto_status=$?
set -e
test "$guard_diff_shared_veto_status" -eq 1
jq -e '.verdict == "VETO" and (.route_crossing_report.suggested_shared_actions | index("shared.manual_toggle_contract"))' <<<"$guard_diff_shared_veto" >/dev/null
cat >"$tmp_atlas_repo/version.diff" <<'EOF_DIFF'
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
diff --git a/Cargo.lock b/Cargo.lock
--- a/Cargo.lock
+++ b/Cargo.lock
diff --git a/VERSIONING.md b/VERSIONING.md
--- a/VERSIONING.md
+++ b/VERSIONING.md
diff --git a/extension/gnome/metadata.json b/extension/gnome/metadata.json
--- a/extension/gnome/metadata.json
+++ b/extension/gnome/metadata.json
diff --git a/extension/gnome/prefs.js b/extension/gnome/prefs.js
--- a/extension/gnome/prefs.js
+++ b/extension/gnome/prefs.js
diff --git a/extension/gnome/settings.js b/extension/gnome/settings.js
--- a/extension/gnome/settings.js
+++ b/extension/gnome/settings.js
diff --git a/extension/gnome/tray_support.js b/extension/gnome/tray_support.js
--- a/extension/gnome/tray_support.js
+++ b/extension/gnome/tray_support.js
EOF_DIFF
guard_diff_version="$("$guard_diff" "$atlas_path" --action-id "shared.version_bump_contract" --diff "$tmp_atlas_repo/version.diff" --format json)"
jq -e '.verdict == "PASS" and .safe_to_edit == true and .reason == "version_bump_contract_pass" and .route_crossing_report.contract_scope == "version metadata only" and .route_crossing_report.decision == "allowed by shared.version_bump_contract" and .version_bump.consistent == true' <<<"$guard_diff_version" >/dev/null
cat >"$tmp_atlas_repo/version-plus-code.diff" <<'EOF_DIFF'
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
diff --git a/src/runtime/manual_trigger_runtime.rs b/src/runtime/manual_trigger_runtime.rs
--- a/src/runtime/manual_trigger_runtime.rs
+++ b/src/runtime/manual_trigger_runtime.rs
EOF_DIFF
set +e
guard_diff_version_veto="$("$guard_diff" "$atlas_path" --action-id "shared.version_bump_contract" --diff "$tmp_atlas_repo/version-plus-code.diff" --format json)"
guard_diff_version_veto_status=$?
set -e
test "$guard_diff_version_veto_status" -eq 1
jq -e '.verdict == "VETO" and .reason == "version_bump_scope_violation" and (.version_bump.scope_violations | index("src/runtime/manual_trigger_runtime.rs"))' <<<"$guard_diff_version_veto" >/dev/null
cat >"$tmp_atlas_repo/extension/gnome/settings.js" <<'EOF_ATLAS_VERSION'
export const APP_VERSION = '1.2.2';
const label = new Gtk.Label({label: `Lay ${APP_VERSION}`});
EOF_ATLAS_VERSION
set +e
guard_diff_version_watch="$("$guard_diff" "$atlas_path" --action-id "shared.version_bump_contract" --diff "$tmp_atlas_repo/version.diff" --format json)"
guard_diff_version_watch_status=$?
set -e
test "$guard_diff_version_watch_status" -eq 3
jq -e '.verdict == "WATCH" and .reason == "version_bump_inconsistent" and (.version_bump.violations | length) > 0' <<<"$guard_diff_version_watch" >/dev/null
cat >"$tmp_atlas_repo/extension/gnome/settings.js" <<'EOF_ATLAS_VERSION'
export const APP_VERSION = '1.2.3';
const label = new Gtk.Label({label: `Lay ${APP_VERSION}`});
EOF_ATLAS_VERSION
tmp_diff_source_repo="$(mktemp -d)"
git -C "$tmp_diff_source_repo" init -q
cat >"$tmp_diff_source_repo/from-other-repo.diff" <<'EOF_DIFF'
diff --git a/src/bin/lay_ibus_engine.rs b/src/bin/lay_ibus_engine.rs
--- a/src/bin/lay_ibus_engine.rs
+++ b/src/bin/lay_ibus_engine.rs
EOF_DIFF
set +e
guard_diff_source_watch="$("$guard_diff" "$atlas_path" --action-id "ime.show_candidate" --diff "$tmp_diff_source_repo/from-other-repo.diff" --format json)"
guard_diff_source_status=$?
set -e
test "$guard_diff_source_status" -eq 3
jq -e '.verdict == "WATCH" and .reason == "diff_source_repo_mismatch" and .safe_to_edit == false and .diff_source.mismatch == true' <<<"$guard_diff_source_watch" >/dev/null
rm -rf "$tmp_diff_source_repo"
set +e
release_json="$("$release_gate" "$atlas_path" --format json)"
release_status=$?
set -e
test "$release_status" -eq 3
jq -e '.mode == "release-gate" and .verdict == "WATCH" and (.routes | index("ime-display-flow"))' <<<"$release_json" >/dev/null
guard_boundary_json="$("$guard_action" "$atlas_path" --symptom "IME not visible" --action-id "ime.activate_engine" --boundary-economics --format json)"
jq -e '.boundary_decision.verdict == "KEEP" and .boundary_decision.principle == "NO_EVIDENCE_NO_CUT"' <<<"$guard_boundary_json" >/dev/null
jq -e '.boundary_economics.boundary_decision.verdict == .boundary_decision.verdict and .boundary_economics.scope == "route-atlas-action"' <<<"$guard_boundary_json" >/dev/null
profile_json="$("$profile_guards" "$tmp_atlas_repo" --iterations 2 --build-iterations 1 --full-iterations 1 --format json)"
jq -e '.mode == "guard-profile" and .avg_ms.build_atlas >= 0 and .avg_ms.guard_action >= 0 and .avg_ms.guard_diff >= 0 and .avg_ms.serve_guard_action >= 0 and .avg_ms.serve_guard_diff >= 0 and .avg_ms.serve_guard_combined_per_request >= 0 and .avg_ms.map_code_repo >= 0 and .avg_ms.dogfood_refactor >= 0' <<<"$profile_json" >/dev/null
printf '{"command":"guard_action","atlas":"%s","symptom":"IME not visible","action_id":"ime.activate_engine"}\n{"command":"guard_diff","atlas":"%s","action_id":"ime.show_candidate","diff":"diff --git a/src/bin/lay_ibus_engine.rs b/src/bin/lay_ibus_engine.rs\\n--- a/src/bin/lay_ibus_engine.rs\\n+++ b/src/bin/lay_ibus_engine.rs\\n"}\n' "$atlas_path" "$atlas_path" >"$tmp_atlas_repo/serve-guards.jsonl"
serve_guards_json="$(nanda-structural-gate/scripts/nanda-serve <"$tmp_atlas_repo/serve-guards.jsonl")"
grep -q '"mode":"guard-action"' <<<"$serve_guards_json"
grep -q '"mode":"guard-diff"' <<<"$serve_guards_json"
grep -q 'SERVE_ATLAS_HIT' <<<"$serve_guards_json"
rm -rf "$tmp_atlas_repo"
tmp_boundary_repo="$(mktemp -d)"
mkdir -p "$tmp_boundary_repo/src/bin/lay_daemon" "$tmp_boundary_repo/src/bin/lay_ibus_engine" "$tmp_boundary_repo/src/nanda" "$tmp_boundary_repo/tests"
cat >"$tmp_boundary_repo/src/runtime_one.rs" <<'EOF_BOUNDARY'
static STATE: std::sync::OnceLock<u8> = std::sync::OnceLock::new();
pub(crate) fn handle_manual_trigger_runtime() { let _ = STATE.get(); }
fn route_private() {}
fn route_private_two() {}
EOF_BOUNDARY
boundary_keep_json="$("$boundary_economics" "$tmp_boundary_repo/src/runtime_one.rs" --route runtime-flow --owner src::runtime_one.rs --format json)"
jq -e '.boundary_decision.verdict == "KEEP"' <<<"$boundary_keep_json" >/dev/null
jq -e '.boundary_field_records.version == "boundary-field-records-v1" and .boundary_field_records.owner == "field_core::boundary" and .boundary_field_records.record_count > 0 and .boundary_field_records.sample[0].kind == "structural_triad"' <<<"$boundary_keep_json" >/dev/null
jq -e '.boundary_field_pass.version == "boundary-field-pass-admission-v1" and .boundary_field_pass.field_pass.verdict == "PASS" and .field_equivalence.field_not_more_permissive == true and .boundary_field_engine.selected_verdict == "KEEP" and .owner_gravity.verdict_hint == "OWNER_STABLE" and .boundary_energy.version == "boundary-energy-v1"' <<<"$boundary_keep_json" >/dev/null
jq -e '.boundary_center.version == "boundary-center-v1-read-only" and .boundary_center.read_only == true and .boundary_center.decision_affects_safe_to_edit == false and .boundary_center.center_contract.version == "field-center-contract-v1" and .boundary_center.center_contract.center_kind == "boundary" and .boundary_center.center_contract.read_only == true and .boundary_center.center_contract.decision_affects_authority == false and .boundary_center.verdict_hint == "CENTER_STABLE_SAFE_LOCAL_EDIT" and .boundary_center.center_gap > 0' <<<"$boundary_keep_json" >/dev/null
cat >"$tmp_boundary_repo/src/bin/lay_daemon/thin_wrapper.rs" <<'EOF_BOUNDARY'
pub fn forward_candidate() { owner_candidate(); }
fn owner_candidate() {}
EOF_BOUNDARY
boundary_merge_json="$("$boundary_economics" "$tmp_boundary_repo/src/bin/lay_daemon/thin_wrapper.rs" --route runtime-flow --owner src::bin::lay_daemon --format json || true)"
jq -e '.boundary_decision.verdict == "MERGE_CANDIDATE" and (.boundary_decision.repair | index("merge wrapper into owner module"))' <<<"$boundary_merge_json" >/dev/null
cat >"$tmp_boundary_repo/src/bin/lay_ibus_engine/mixed.rs" <<'EOF_BOUNDARY'
pub fn ime_display_candidate() { score_candidate(); }
fn score_candidate() {}
EOF_BOUNDARY
cat >"$tmp_boundary_repo/src/nanda/scoring.rs" <<'EOF_BOUNDARY'
pub fn nanda_score_candidate() { score_candidate(); }
fn score_candidate() {}
EOF_BOUNDARY
cat >"$tmp_boundary_repo/tests/mixed_test.rs" <<'EOF_BOUNDARY'
#[test] fn mixed_boundary_test() { assert!(true); }
EOF_BOUNDARY
set +e
boundary_split_json="$("$boundary_economics" "$tmp_boundary_repo" --format json)"
boundary_split_status=$?
set -e
test "$boundary_split_status" -eq 1
jq -e '.boundary_decision.verdict == "SPLIT_STRONG" and .boundary_decision.evidence.foreign_pull != []' <<<"$boundary_split_json" >/dev/null
jq -e '.boundary_field_records.record_count > 0 and .boundary_field_records.foreign_pull_records > 0 and .boundary_field_records.call_edge_records > 0' <<<"$boundary_split_json" >/dev/null
jq -e '.boundary_field_pass.field_pass.verdict == "VETO" and .field_equivalence.field_not_more_permissive == true and .boundary_field_engine.selected_verdict == "VETO" and .owner_gravity.verdict_hint == "OWNER_CONFLICT" and .boundary_energy.verdict_hint == "NO_CUT_REPAIR_OWNER_OR_ROUTE"' <<<"$boundary_split_json" >/dev/null
jq -e '.boundary_center.verdict_hint == "CENTER_FOREIGN_PULL_REVIEW" and .boundary_center.foreign_route_mass > 0 and .boundary_center.read_only == true' <<<"$boundary_split_json" >/dev/null
tmp_boundary_pressure_repo="$(mktemp -d)"
mkdir -p "$tmp_boundary_pressure_repo/src/shared_owner/core"
cat >"$tmp_boundary_pressure_repo/src/shared_owner/core/nanda_support.rs" <<'EOF_BOUNDARY'
pub fn nanda_support() {}
EOF_BOUNDARY
for i in 1 2 3 4 5; do
  cat >"$tmp_boundary_pressure_repo/src/shared_owner/core/nanda_api_$i.rs" <<EOF_BOUNDARY
pub fn nanda_api_$i() {}
EOF_BOUNDARY
done
cat >"$tmp_boundary_pressure_repo/src/shared_owner/core/runtime_call.rs" <<'EOF_BOUNDARY'
pub fn runtime_call() { nanda_support(); }
EOF_BOUNDARY
for i in 1 2 3; do
  cat >"$tmp_boundary_pressure_repo/src/shared_owner/core/runtime_api_$i.rs" <<EOF_BOUNDARY
pub fn runtime_api_$i() {}
EOF_BOUNDARY
done
boundary_pressure_json="$("$boundary_economics" "$tmp_boundary_pressure_repo" --format json || true)"
jq -e '.boundary_core.owner == "field_core::boundary" and .boundary_core.commands_are_wrappers == true and .boundary_decision.verdict == "WATCH" and .boundary_decision.safe_to_edit == false and (.boundary_decision.reason | contains("repo-wide route pressure")) and (.boundary_decision.repair | index("rerun boundary economics with --atlas, --route, and --owner before cutting"))' <<<"$boundary_pressure_json" >/dev/null
boundary_find_json="$("$boundary_economics" "$tmp_boundary_repo" --find-refactors --format json)"
jq -e '.mode == "boundary-refactor-finder" and .boundary_core.owner == "field_core::boundary" and .boundary_core.commands_are_wrappers == true and .safe_to_edit == false and .ranking_policy.no_size_only_split == true and (.refactor_candidates | length) > 0' <<<"$boundary_find_json" >/dev/null
boundary_pressure_dogfood_json="$("$dogfood" "$tmp_boundary_pressure_repo" --refactor-plan --boundary-economics --format json || true)"
jq -e '.agent_decision.action == "REVIEW_REQUIRED" and .agent_decision.safe_to_edit == false and .agent_decision.boundary_economics_verdict == "WATCH" and .boundary_economics.boundary_decision.verdict == "WATCH"' <<<"$boundary_pressure_dogfood_json" >/dev/null
boundary_atlas_path="$tmp_boundary_repo/.nanda/route-atlas.json"
"$build_atlas" "$tmp_boundary_repo" --out "$boundary_atlas_path" --format json >/dev/null
boundary_scoped_json="$("$boundary_economics" "$tmp_boundary_repo" --atlas "$boundary_atlas_path" --route ime-display-flow --owner src::bin::lay_ibus_engine --format json || true)"
jq -e '.scope == "route-scoped" and (.boundary_decision.verdict | IN("KEEP","MERGE_CANDIDATE")) and (.boundary_decision.evidence.files | all(contains("lay_ibus_engine"))) and .boundary_decision.evidence.foreign_pull == [] and (.boundary_decision.required_tests | length) <= 5' <<<"$boundary_scoped_json" >/dev/null
jq -e '.boundary_field_records.version == "boundary-field-records-v1" and .boundary_field_records.file_records == (.boundary_decision.evidence.files | length)' <<<"$boundary_scoped_json" >/dev/null
jq -e '.field_equivalence.field_not_more_permissive == true and (.boundary_field_engine.selected_verdict | IN("KEEP","MERGE_CANDIDATE","WATCH"))' <<<"$boundary_scoped_json" >/dev/null
jq -e '.boundary_center.version == "boundary-center-v1-read-only" and .boundary_center.decision_affects_safe_to_edit == false and .boundary_center.route_center != null' <<<"$boundary_scoped_json" >/dev/null
boundary_wrong_owner_json="$("$boundary_economics" "$tmp_boundary_repo" --atlas "$boundary_atlas_path" --route ime-display-flow --owner WrongOwner --format json || true)"
jq -e '.scope == "route-scoped" and .boundary_decision.verdict == "WATCH" and .boundary_decision.safe_to_edit == false and .boundary_decision.evidence.owner_filter.requested == true and .boundary_decision.evidence.owner_filter.matched == false and .boundary_decision.allowed_files == []' <<<"$boundary_wrong_owner_json" >/dev/null
boundary_veto_json="$("$boundary_economics" "$tmp_boundary_repo" --route ime-display-flow --owner src::bin::lay_ibus_engine --format json || true)"
jq -e '.boundary_decision.verdict == "VETO"' <<<"$boundary_veto_json" >/dev/null
tmp_watch_repo="$(mktemp -d)"
touch "$tmp_watch_repo/README.md"
boundary_watch_json="$("$boundary_economics" "$tmp_watch_repo" --format json || true)"
jq -e '.boundary_decision.verdict == "WATCH"' <<<"$boundary_watch_json" >/dev/null
dogfood_boundary_json="$("$dogfood" "$tmp_boundary_repo" --refactor-plan --boundary-economics --format json || true)"
jq -e '.boundary_economics.boundary_decision.verdict | IN("SPLIT_STRONG","SPLIT_WEAK","VETO","KEEP","MERGE_CANDIDATE","WATCH")' <<<"$dogfood_boundary_json" >/dev/null
rm -rf "$tmp_boundary_repo" "$tmp_boundary_pressure_repo" "$tmp_watch_repo"

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
jq -e '.field_runtime_cutover_guard.version == "packed-field-runtime-cutover-guard-v1" and .field_runtime_cutover_guard.packed_field_record_view == "packed-field-record-view-v1" and .field_runtime_cutover_guard.bench_evidence_present == true and .field_runtime_cutover_guard.field_core_as_packed_sole_engine_allowed == true and .field_runtime_cutover_guard.field_core_as_sole_engine_allowed == false' <<<"$bench6m_json" >/dev/null
jq -e '.benchmarks.density.iterations == 100 and .benchmarks.density.kernel == "packed_density_probe" and .benchmarks.density.triads_in_memory == 8 and .benchmarks.density.accuracy == 1 and .benchmarks.density.false_positive == 0' <<<"$bench6m_json" >/dev/null
bench6m_active65k_json="$("$bench6m" --mode active-65k --active-65k-iterations 1 --format json)"
jq -e '.benchmarks.active_65k.mode == "active-65k-full-field" and .benchmarks.active_65k.active_records == 65536 and .benchmarks.active_65k.records_scanned_per_iteration == 131072' <<<"$bench6m_active65k_json" >/dev/null
jq -e '.benchmarks.active_65k.runtime_contract.state == "ACTIVE_65K_READY" and .benchmarks.active_65k.runtime_contract.full_active_scan == true and .benchmarks.active_65k.runtime_contract.streaming_discovery == true and .benchmarks.active_65k.runtime_contract.proof_rescan == true' <<<"$bench6m_active65k_json" >/dev/null
jq -e '.benchmarks.active_65k.workspace_bounded == true and .benchmarks.active_65k.no_per_record_score_arrays == true and .benchmarks.active_65k.claim_boundary.llm_ready == false and .benchmarks.active_65k.claim_boundary.nonlinear_memory_proven == false' <<<"$bench6m_active65k_json" >/dev/null
jq -e '.benchmarks.active_65k.authority.proof_rescan_completed == true and .benchmarks.active_65k.authority.candidate_without_proof_can_answer == false and .benchmarks.active_65k.authority.candidate_without_proof_can_write_memory == false' <<<"$bench6m_active65k_json" >/dev/null
bench6m_active65k_discovery_json="$("$bench6m" --mode active-65k-discovery --active-65k-iterations 1 --format json)"
jq -e '.benchmarks.active_65k_discovery.mode == "active-65k-discovery" and .benchmarks.active_65k_discovery.active_records == 65536 and .benchmarks.active_65k_discovery.records_scanned_per_iteration == 65536 and .benchmarks.active_65k_discovery.proof_records_scanned == 0' <<<"$bench6m_active65k_discovery_json" >/dev/null
jq -e '.benchmarks.active_65k_discovery.authority.state == "PROOF_REQUIRED" and .benchmarks.active_65k_discovery.authority.safe_to_answer == false and .benchmarks.active_65k_discovery.authority.candidate_without_proof_can_answer == false and .benchmarks.active_65k_discovery.authority.candidate_without_proof_can_write_memory == false' <<<"$bench6m_active65k_discovery_json" >/dev/null
jq -e '.benchmarks.active_65k_discovery.claim_boundary.interactive_discovery_ready == true and .benchmarks.active_65k_discovery.claim_boundary.answer_authority_ready == false and .benchmarks.active_65k_discovery.claim_boundary.proof_required == true and .benchmarks.active_65k_discovery.claim_boundary.full_proof_not_cut == true' <<<"$bench6m_active65k_discovery_json" >/dev/null
field_plate_dir="$(mktemp -d)"
field_plate_path="$field_plate_dir/active65k.plate.json"
field_plate_svg="$field_plate_dir/active65k.svg"
"$field_plate" --help | grep -q "Usage: nanda field-plate"
field_plate_build_json="$("$field_plate" build --out "$field_plate_path" --format json)"
jq -e '.mode == "field-plate-build" and .verdict == "FIELD_PLATE_BUILT" and .safe_to_use_kernel == true and .plate.records == 65536 and .plate.wave_dim == 1024 and .plate.claim_boundary.visual_render_is_not_authority == true' <<<"$field_plate_build_json" >/dev/null
test -s "$field_plate_path"
field_plate_check_json="$("$field_plate" check --plate "$field_plate_path" --format json)"
jq -e '.mode == "field-plate-check" and .comparison.verdict == "FIELD_PLATE_MATCH" and .comparison.safe_to_use_kernel == true and .comparison.exact_match == true and (.comparison.mismatches | length) == 0' <<<"$field_plate_check_json" >/dev/null
field_plate_render_json="$("$field_plate" render --plate "$field_plate_path" --out "$field_plate_svg" --format json)"
jq -e '.mode == "field-plate-render" and .verdict == "FIELD_PLATE_RENDERED" and .visual_render_is_not_authority == true' <<<"$field_plate_render_json" >/dev/null
test -s "$field_plate_svg"
grep -q "NANDA Field Plate" "$field_plate_svg"
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
jq -e '.field_runtime_cutover_guard.bench_evidence_present == true and .field_runtime_cutover_guard.field_core_as_packed_sole_engine_allowed == true and .field_runtime_cutover_guard.field_core_as_sole_engine_allowed == false' <<<"$bench6m_hot_cycle_json" >/dev/null
bench6m_density_json="$("$bench6m" --mode density --support-build-iterations 10 --triads 16 --format json)"
jq -e '.benchmarks.replay == null and .benchmarks.projection == null and .benchmarks.density.iterations == 10 and .benchmarks.density.triads_in_memory == 16 and .benchmarks.density.accuracy == 1 and .benchmarks.density.false_positive == 0' <<<"$bench6m_density_json" >/dev/null
jq -e '.field_runtime_cutover_guard.bench_evidence_present == false and .field_runtime_cutover_guard.field_core_as_sole_engine_allowed == false' <<<"$bench6m_density_json" >/dev/null
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
tmp_field_report="$(mktemp -d)"
cat >"$tmp_field_report/structural.json" <<'EOF_FIELD'
{
  "mode": "search",
  "query": {"source": "auto_query_triads", "text": "route check"},
  "peaks": [{"peak": "runtime", "score": 0.88}],
  "peak_margin": 0.22,
  "peak_decision": {"state": "FOCUSED", "safe_to_answer": true},
  "field_state_machine": {"state": "FIELD_FOCUSED", "action": "draft_with_evidence"},
  "structural_map": {
    "route_field": {"routes": {"runtime": {}}},
    "group_centroids": {"source": {}, "candidate": {}},
    "structural_energy": {"route_coherence": 0.91},
    "foreign_pull": []
  },
  "supporting_triads": [{"id": "t1"}]
}
EOF_FIELD
field_structural_json="$("$field_report" --from "$tmp_field_report/structural.json" --format json)"
jq -e '.version == "unified-field-v1-readonly" and .family == "structural" and .peak.target == "runtime" and .peak.safe_to_answer == true and .claim_boundary.no_behavior_change == true and (.query.signature | type == "string" and length == 16) and .compute_probe.version == "unified-field-compute-v1"' <<<"$field_structural_json" >/dev/null
cat >"$tmp_field_report/packed.json" <<'EOF_FIELD'
{
  "mode": "pack6m",
  "packed_runtime": {"triads": 15000, "routes": 12, "groups": 64},
  "packed_support": {"support_count": 7, "anti_count": 2, "net_dot": 144},
  "packed_lanes": {"count": 2, "delta": 64, "suppressed_peak": "false-route"},
  "packed_peak": {"target": "route-7", "score": 0.77, "cosine": 0.41},
  "peak_decision": {"state": "PACKED_FOCUSED", "safe_to_answer": false}
}
EOF_FIELD
field_packed_json="$("$field_report" --from "$tmp_field_report/packed.json" --format json)"
jq -e '.family == "packed" and .basis.memory_budget_class == "hot_cache_budgeted" and .anti_wave.active == true and .compatibility.hot_loop_unchanged == true and (.query.signature | type == "string" and length == 16) and .compute_probe.dim == 1024' <<<"$field_packed_json" >/dev/null
cat >"$tmp_field_report/cognitive.json" <<'EOF_FIELD'
{
  "roadmap_block": "v-test",
  "verdict": "LLMWAVE_REVIEW",
  "input_text": "what does declaration require",
  "metrics": {"record_count": 3, "schema_count": 2, "coherence": 0.74},
  "claim_boundary": {"llm_ready": false, "nonlinear_memory_proven": false, "cache_only_execution_proven": false, "safe_to_answer": false},
  "field_after_anti": {"state": "review"}
}
EOF_FIELD
field_cognitive_json="$("$field_report" --from "$tmp_field_report/cognitive.json" --format json)"
jq -e '.family == "cognitive" and .basis.axis_policy == "l2_surface_l3_schema_axes" and .claim_boundary.not_llm_ready == true and .claim_boundary.not_nonlinear_memory_proof == true and (.query.signature | type == "string" and length == 16) and .compute_probe.version == "unified-field-compute-v1"' <<<"$field_cognitive_json" >/dev/null
field_audit_json="$("$field_audit" --format json)"
jq -e '.mode == "unified-field-audit" and .version == "unified-field-pass-v1" and .overall_state == "FIELD_CORE_SOLE_ENGINE_ACTIVE_LLM_NOT_READY" and .acceptance.one_field_pass == true and .acceptance.field_core_as_semantic_engine == true and .acceptance.feedback_memory_delta_unified == true and .acceptance.semantic_equivalence_gate == true and .acceptance.structural_dual_run_active == true and .acceptance.structural_cutover_eval_ready == true and .acceptance.structural_cutover_suite_available == true and .acceptance.structural_cutover_suite_pass == true and .acceptance.structural_field_core_as_sole_engine == true and .families[0].sole_engine == true and .families[1].sole_engine == true and .families[2].sole_engine == true and .structural_cutover_suite.suite == "structural-standard" and .structural_cutover_suite.acceptance.cases_checked == 4 and .structural_cutover_suite.acceptance.structural_cutover_suite_pass == true and .acceptance.packed_dual_run_active == true and .acceptance.packed_hot_core_exception == false and .acceptance.packed_field_core_as_sole_engine == true and .acceptance.packed_field_record_view == true and .acceptance.cognitive_dual_run_active == true and .acceptance.cognitive_field_core_as_sole_engine == true and .acceptance.cognitive_claim_guard_blocks_llm == true and .acceptance.unified_lens_contract == true and .acceptance.unified_anti_wave_contract == true and .acceptance.unified_memory_delta_store == true and .acceptance.route_scoped_extraction_required == false and .acceptance.field_core_as_sole_engine == true and .acceptance.llm_ready == false and .acceptance.nonlinear_memory_proven == false and (.next_required_steps | length) == 0' <<<"$field_audit_json" >/dev/null
jq -e '.sole_engine_contract.version == "unified-field-sole-engine-v1" and .sole_engine_contract.big_pipelines == .sole_engine_contract.field_core_backed_pipelines and .sole_engine_contract.local_physics_copies_allowed == false and .sole_engine_contract.field_core_as_sole_engine == true and .acceptance.sole_engine_registry == true and .acceptance.big_pipelines_registered == .acceptance.field_core_backed_pipelines and (.sole_engine_contract.pipeline_consumers[] | select(.pipeline == "llmwave-lens-scan") | .uses_field_pass == true and .local_physics_allowed == false)' <<<"$field_audit_json" >/dev/null
jq -e '.field_engine_contract.version == "unified-field-engine-contract-v1" and .field_engine_contract.policy_owner == "field_core::engine::FieldEngineDecision" and .field_engine_contract.families_checked == 3 and .field_engine_contract.global_sole_engine == true and .field_engine_contract.llm_ready == false and .field_engine_contract.nonlinear_memory_proven == false and .acceptance.three_family_engine_contract == true and .acceptance.field_engine_policy_in_field_core == true and .acceptance.structural_cutover_mode_available == true and .field_engine_contract.structural.cutover_allowed == true and .field_engine_contract.structural.cutover_mode == "default" and .field_engine_contract.structural.structural_sole_engine == true and .field_engine_contract.structural.scope == "structural" and .acceptance.packed_field_engine_guard == true and .acceptance.packed_cutover_blocked_by_hot_guard == false and .field_engine_contract.packed.cutover_allowed == true and .field_engine_contract.packed.blocked_by == null and .field_engine_contract.packed.packed_sole_engine == true and .field_engine_contract.packed.scope == "packed" and .acceptance.cognitive_field_engine_guard == true and .acceptance.cognitive_cutover_blocked_by_claim_guard == false and .field_engine_contract.cognitive.cutover_allowed == true and .field_engine_contract.cognitive.cognitive_sole_engine == true and .field_engine_contract.cognitive.scope == "cognitive" and .field_engine_contract.cognitive.chat_engine == false and .field_engine_contract.cognitive.llm_ready == false' <<<"$field_audit_json" >/dev/null
jq -e '.local_physics_inventory.version == "field-local-physics-audit-v1" and .local_physics_inventory.source_scan_available == true and .local_physics_inventory.verdict == "FIELD_CORE_SOLE_ENGINE_WITH_REVIEW_DEBT" and .local_physics_inventory.totals.local_physics_candidates == 0 and .local_physics_inventory.totals.domain_fixture_readouts > 0 and .local_physics_inventory.totals.structural_legacy_readouts > 0 and .acceptance.local_physics_inventory_present == true and .acceptance.local_physics_candidates == 0' <<<"$field_audit_json" >/dev/null
jq -e '.field_operation_contract.version == "unified-field-operation-contract-v1" and .field_operation_contract.peak_owner == "field_core::peak::FieldPeakResult" and .field_operation_contract.coherence_owner == "field_core::coherence::FieldCoherenceResult" and .field_operation_contract.anti_wave_owner == "field_core::anti_wave::FieldAntiWaveEffect" and .field_operation_contract.readout_owner == "field_core::readout::FieldReadoutResult" and .field_operation_contract.local_path_owner == "field_core::readout::FieldLocalPathResult" and .field_operation_contract.structural_decision_uses_field_core == true and .acceptance.field_core_owns_peak_contract == true and .acceptance.field_core_owns_coherence_contract == true and .acceptance.field_core_owns_anti_wave_contract == true and .acceptance.field_core_owns_readout_contract == true and .acceptance.field_core_owns_local_path_contract == true and .acceptance.structural_decision_uses_field_core == true' <<<"$field_audit_json" >/dev/null
field_equivalence_json="$("$field_equivalence" --structural-from "$tmp_field_report/structural.json" --packed-from "$tmp_field_report/packed.json" --cognitive-from "$tmp_field_report/cognitive.json" --format json)"
jq -e '.mode == "unified-field-equivalence" and .version == "unified-field-pass-v1" and .acceptance.equivalent_contract == true and .acceptance.all_have_field_pass == true and .acceptance.all_have_memory_delta == true and .acceptance.field_core_as_sole_engine == false and .acceptance.llm_ready == false' <<<"$field_equivalence_json" >/dev/null
printf '%s\n' "$trap_search_json" >"$tmp_field_report/cutover-focused.json"
printf '%s\n' "$noisy_search_json" >"$tmp_field_report/cutover-contested.json"
printf '%s\n' "$polarity_stop_json" >"$tmp_field_report/cutover-reversed.json"
printf '%s\n' "$negative_lanes_json" >"$tmp_field_report/cutover-thin.json"
field_cutover_json="$("$field_cutover" \
  --structural-case "$tmp_field_report/cutover-focused.json" \
  --structural-case "$tmp_field_report/cutover-contested.json" \
  --structural-case "$tmp_field_report/cutover-reversed.json" \
  --structural-case "$tmp_field_report/cutover-thin.json" \
  --format json)"
jq -e '.mode == "unified-field-cutover-suite" and .version == "unified-field-runtime-v1" and .family == "structural" and .acceptance.cases_checked == 4 and .acceptance.structural_cutover_suite_pass == true and .acceptance.all_peak_match == true and .acceptance.all_state_family_match == true and .acceptance.all_not_more_permissive == true and .acceptance.field_core_as_structural_engine_candidate == true and .acceptance.field_core_as_structural_sole_engine_allowed == true and .acceptance.field_core_as_sole_engine_allowed == false and .claim_boundary.global_sole_engine == false and .claim_boundary.structural_only_sole_engine == true and .claim_boundary.requires_explicit_follow_up_cutover == false' <<<"$field_cutover_json" >/dev/null
field_cutover_suite_json="$("$field_cutover" --suite structural-standard --format json)"
jq -e '.mode == "unified-field-cutover-suite" and .suite == "structural-standard" and .version == "unified-field-runtime-v1" and .family == "structural" and .acceptance.cases_checked == 4 and .acceptance.structural_cutover_suite_pass == true and .acceptance.all_peak_match == true and .acceptance.all_state_family_match == true and .acceptance.all_not_more_permissive == true and .acceptance.field_core_as_structural_engine_candidate == true and .acceptance.field_core_as_structural_sole_engine_allowed == true and .acceptance.field_core_as_sole_engine_allowed == false and .claim_boundary.global_sole_engine == false and .claim_boundary.structural_only_sole_engine == true and .claim_boundary.requires_explicit_follow_up_cutover == false' <<<"$field_cutover_suite_json" >/dev/null

"$search" "$root/examples/triad-packet.interference-search.json" --query "runtime route" --format json >"$tmp_field_report/live-search.json"
live_structural_field="$("$field_report" --from "$tmp_field_report/live-search.json" --format json)"
jq -e '.family == "structural" and (.query.signature | type == "string" and length == 16) and .compute_probe.version == "unified-field-compute-v1"' <<<"$live_structural_field" >/dev/null

"$pack6m" "$root/examples/triad-packet.pack6m-replay-waw.json" --query "route replay" --format json >"$tmp_field_report/live-pack6m.json"
live_packed_field="$("$field_report" --from "$tmp_field_report/live-pack6m.json" --format json)"
jq -e '.family == "packed" and (.query.signature | type == "string" and length == 16) and .compute_probe.dim == 1024' <<<"$live_packed_field" >/dev/null

"$llmwave_big" query-wave --text "Has customs cleared the goods?" --format json >"$tmp_field_report/live-llmwave-big.json"
live_cognitive_field="$("$field_report" --from "$tmp_field_report/live-llmwave-big.json" --format json)"
jq -e '.family == "cognitive" and (.query.signature | type == "string" and length == 16) and .claim_boundary.not_llm_ready == true and .compute_probe.version == "unified-field-compute-v1"' <<<"$live_cognitive_field" >/dev/null
rm -rf "$tmp_field_report"
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
"$boundary_economics" --help | grep -q "Usage: nanda boundary-economics"
"$build_atlas" --help | grep -q "Usage: nanda build-atlas"
"$guard_action" --help | grep -q "Usage: nanda guard-action"
"$guard_diff" --help | grep -q "Usage: nanda guard-diff"
"$profile_guards" --help | grep -q "Usage: nanda profile-guards"
"$release_gate" --help | grep -q "Usage: nanda release-gate"
"$field_report" --help | grep -q "Usage: nanda field-report"
"$field_audit" --help | grep -q "Usage: nanda field-audit"
"$field_equivalence" --help | grep -q "Usage: nanda field-equivalence"
"$field_cutover" --help | grep -q "Usage: nanda field-cutover"
"$split_packet" --help | grep -q "Usage: nanda split"
"$reporter" --format md --title "Smoke Markdown Report" --domain code --overall "$root/examples/triads.watch-large.md" --route code:"$root/examples/triads.code-flow.md" >/dev/null || test "$?" -eq 3

echo "ok"
