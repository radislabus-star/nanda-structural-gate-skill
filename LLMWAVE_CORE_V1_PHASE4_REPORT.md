# LLMWave Core V1 Phase 4 Report

Status: Phase 4 nonlinear-memory proof gate recorded.
Created: 2026-06-25.
Machine report: `nanda-llmwave-big core-v1-nonlinear-proof --format json`.

## Scope

Phase 4 implements the Core V1 nonlinear-memory proof gate. It does not force a
PASS. It combines:

```text
Phase 3 memory-writer evidence
fixed-basis density ladder
linear baseline comparison
role/false-positive bounds
held-out/external/noise/leakage requirements
```

## Current Verdict

```text
verdict = CORE_V1_NONLINEAR_MEMORY_CANDIDATE_BLOCKED
nonlinear_memory_candidate = true
nonlinear_memory_proven = false
llm_ready = false
```

This is the honest state: the current fixture shows a density candidate, but
the final proof remains blocked.

## Passed Gates

```text
writer_beats_raw_dictionary_fixture = true
raw_dictionary_is_not_primary_memory = true
bytes_per_useful_fact_falls_at_three_scale_points = true
scale_ladder_ready = true
linear_baseline_compared = true
role_error_rate_bounded = true
false_positive_rate_bounded = true
```

## Blocked Gates

```text
heldout_quality_bound_to_writer = false
external_corpus_present = false
near_duplicate_leakage_control_present = false
broad_noise_eval_present = false
selected_policy_proven = false
```

## Key Metrics

```text
max_facts = 100000
scale_points = 5
bytes_fall_points = 4
amortized_win_point = 10
standalone_break_even_point = 1000
min_heldout_pass_rate = 0.9827
max_role_error_rate = 0.0135
max_false_positive_rate = 0.0212
writer_saving_ratio = 0.1993
```

## Claim Boundary

Phase 4 opens:

```text
nonlinear_memory_proof_gate_implemented = true
nonlinear_memory_candidate = true
```

Phase 4 does not open:

```text
nonlinear_memory_proven = false
llm_ready = false
```

## Commands

Machine JSON:

```bash
nanda-llmwave-big core-v1-nonlinear-proof --format json
```

Markdown:

```bash
nanda-llmwave-big core-v1-nonlinear-proof --format md
```

Text:

```bash
nanda-llmwave-big core-v1-nonlinear-proof --format text
```

## Next Phase

The next phase is:

```text
phase-5-query-wave-v1
```

The proof gate will remain blocked until query, held-out, external corpus,
near-duplicate leakage, and broad noise evidence are bound to the same memory
writer path.
