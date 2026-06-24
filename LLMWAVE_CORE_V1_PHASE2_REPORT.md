# LLMWave Core V1 Phase 2 Report

Status: Phase 2 field-core cutover recorded.
Created: 2026-06-25.
Machine report: `nanda-llmwave-big core-v1-field-cutover --format json`.

## Scope

Phase 2 makes `field_core` the single declared owner for shared field
operations:

```text
peak_detection
coherence_state
verdict
anti_wave
readout
local_path
runtime_dual_run
```

The cutover is intentionally scoped to field operations. It does not claim that
the whole LLMWave Core V1 loop is implemented.

## Families

Three field families are mapped to the same operation contract:

| Family | State | Guard |
|---|---|---|
| `structural-search` | `mapped_through_structural-field-engine-v1` | structural cutover suite, not-more-permissive check |
| `packed-runtime` | `mapped_with_hot_loop_guard` | typed packed decision core, hot-loop guard |
| `llmwave-cognitive` | `mapped_not_chat_engine` | not-LLM, nonlinear-memory, full-field claim guards |

## Claim Boundary

Phase 2 opens only this claim:

```text
field_core_as_sole_field_operations_engine = true
```

Phase 2 does not open:

```text
field_core_as_sole_llmwave_core_engine = false
evidence_bound_answer_ready = false
feedback_learning_ready = false
nonlinear_memory_proven = false
llm_ready = false
```

That distinction matters. `field_core` now owns the shared field physics for
reports and guarded runtime decisions. The full Core V1 loop still needs the
memory writer, query wave, answer surface, verifier, feedback learning, and
baseline eval.

## Phase 2 Exit Criteria

```text
phase_1_contract_present = true
shared_field_operations_present = true
structural_family_mapped = true
packed_family_mapped = true
cognitive_family_mapped = true
claim_boundary_preserved = true
docs_updated = true
```

## Self Gate

Action gate used before the phase:

```bash
nanda-guard-action .nanda/route-atlas.json \
  --symptom "cut over LLMWave Core V1 field physics to field_core" \
  --action-id llmwave.field_core_cutover \
  --boundary-economics \
  --format json
```

The documentation/code diff is a shared architecture-contract diff because it
touches source, docs, runtime skill instructions, and local tests.

## Commands

Machine JSON:

```bash
nanda-llmwave-big core-v1-field-cutover --format json
```

Markdown:

```bash
nanda-llmwave-big core-v1-field-cutover --format md
```

Text:

```bash
nanda-llmwave-big core-v1-field-cutover --format text
```

## Next Phase

The next phase is:

```text
phase-3-memory-writer-v1
```

Phase 3 must implement memory writing without turning the model back into a
flat `token_id -> UTF-8` dictionary as the primary cognitive memory.
