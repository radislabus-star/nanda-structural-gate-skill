# LLMWave Core V1 Phase 12 Report

Phase 12 adds the Core V1 broad eval harness. It runs embedded controls across
the local pipeline and keeps hard claims blocked unless real broad-corpus and
density-proof evidence exists.

## Command

```bash
nanda-llmwave-big core-v1-broad-eval \
  --text "Has customs cleared the goods?" \
  --format json
```

## Verdict

```text
CORE_V1_BROAD_EVAL_HARNESS_READY_NOT_LLM
```

This means the embedded local Core V1 controls pass. It does not mean a broad
LLM is ready, and it does not prove nonlinear memory.

## Embedded Controls

- query wave paraphrase stability;
- retrieval focus;
- schema missing-dependency propagation;
- evidence-bound surface refusal;
- verified local refusal;
- unsupported positive shortcut rejection;
- role-swap rejection;
- feedback changes next pass;
- sleep pass preserves negative lane;
- hard claims remain blocked without broad corpus.

## Fixed Record

Phase 12 introduces `CoreV1EvalCaseRecord32`, a 32-byte eval case record:

- case id
- stage id
- expected id
- observed id
- pass flag
- safety flag
- score
- margin
- false-positive flag
- false-negative flag

## Claim Boundary

```text
broad_eval_harness_v1_implemented = true
local_core_v1_pipeline_ready       = true
safety_controls_ready             = true
real_broad_corpus_loaded           = false
broad_generalization_proven        = false
llm_ready                          = false
nonlinear_memory_proven            = false
```

## Next Required Evidence

```text
core-v1-real-broad-corpus-and-density-proof
```

The next path is not another fixture layer. It must bind Core V1 to real broad
corpus evidence, external held-out questions, density proof, and broad
false-positive controls before any LLM or nonlinear-memory claim can open.
