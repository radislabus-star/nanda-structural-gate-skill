# LLMWave Core V1 Phase 3 Report

Status: Phase 3 memory writer recorded.
Created: 2026-06-25.
Machine report: `nanda-llmwave-big core-v1-memory-writer --format json`.

## Scope

Phase 3 implements the Core V1 memory-writer report over existing typed memory
parts:

```text
schema residuals
surface family refs
evidence pointers / hashes
duplicate rejection
noise rejection
```

This is the first Core V1 layer that answers the earlier design problem:
visible words and facts are not treated as a primary `token_id -> UTF-8`
dictionary. Observed forms are evidence, surface-family material, or exact copy
fallbacks. Schema cognition is written through reusable route/operator/role
shapes plus residual deltas.

## Current Fixture Metrics

From the current deterministic fixture:

```text
raw_dictionary_baseline_bytes = 1219
writer_total_bytes            = 976
writer_saving_ratio           = 0.1993
schema_reuse_ratio            = 3.3333
residual_only_coverage        = 0.9091
residual_saving_ratio         = 0.7083
rejected_duplicate_count      = 2
rejected_noise_count          = 2
```

The surface-family bank is present and has refs, but on this tiny fixture it is
not by itself a nonlinear-memory proof. The combined writer wins because the
schema/residual path reuses repeated relation shapes.

## Memory Write Policy

```text
primary_memory = schema_residuals_plus_surface_family_refs
raw_surface_rule = observed forms are evidence/copy spans, not primary cognition
schema_rule = promote reused route/operator/role shapes
residual_rule = write subject/object/phase/evidence against promoted schema
evidence_rule = store evidence refs/hashes, not full evidence text in hot memory
hot_core_rule = hot core sees ids, phases, hashes, refs, not UTF-8 dictionary
```

## Phase 3 Exit Criteria

```text
residual_write_path_active = true
raw_dictionary_is_not_primary_memory = true
memory_write_report_present = true
surface_family_refs_defined = true
evidence_pointer_fields_defined = true
rejected_duplicate_count_reported = true
rejected_noise_count_reported = true
```

## Claim Boundary

Phase 3 opens only this claim:

```text
residual_write_path_active = true
raw_dictionary_is_not_primary_memory = true
memory_write_report_present = true
```

Phase 3 does not open:

```text
nonlinear_memory_proven = false
llm_ready = false
```

Still blocked by:

```text
phase_4_nonlinear_memory_proof_missing
scale_ladder_not_passed
heldout_quality_not_bound_to_memory_writer
query_wave_v1_missing
answer_verifier_v1_missing
```

## Commands

Machine JSON:

```bash
nanda-llmwave-big core-v1-memory-writer --format json
```

Markdown:

```bash
nanda-llmwave-big core-v1-memory-writer --format md
```

Text:

```bash
nanda-llmwave-big core-v1-memory-writer --format text
```

## Next Phase

The next phase is:

```text
phase-4-nonlinear-memory-proof-v1
```

Phase 4 must test whether the writer's density advantage survives scale,
held-out reasoning, role swaps, false positives, near-duplicate leakage, and
baselines. Until that passes, nonlinear memory remains a candidate, not a proof.
