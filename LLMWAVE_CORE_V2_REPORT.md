# LLMWave Core V2 Report

Core V2 is the next local pipeline after Core V1. Its purpose is to connect the
pieces that were still separate in Core V1: corpus artifact, held-out removal,
route-balanced focus, density economics, local route run, hot packet storage,
and a hard claim gate.

Core V2 is intentionally not a claim that the project is a general LLM. The
expected passing verdict is:

```text
CORE_V2_LOCAL_PIPELINE_READY_NOT_LLM
```

This means the local fixture pipeline is wired and guarded. It still keeps:

```text
llm_ready                    = false
nonlinear_memory_proven      = false
real_broad_corpus_loaded     = false
cache_only_execution_proven  = false
```

## Commands

```bash
nanda-llmwave-big core-v2-contract --format json
nanda-llmwave-big core-v2-corpus --format json
nanda-llmwave-big core-v2-heldout --format json
nanda-llmwave-big core-v2-focus --format json
nanda-llmwave-big core-v2-density --format json
nanda-llmwave-big core-v2-run --text "Has customs cleared the goods?" --format json
nanda-llmwave-big core-v2-pack-hot --format json
nanda-llmwave-big core-v2-claim-gate --format json
```

## Stages

1. `core-v2-contract`
   Records the staged pipeline and fixed packed records:
   `CoreV2FactRecord48`, `CoreV2HeldoutCaseRecord32`,
   `CoreV2FocusRecord32`, `CoreV2DensityRecord32`, and
   `CoreV2HotRecord32`.

2. `core-v2-corpus`
   Builds a small embedded public-safe relation fixture. It contains no private
   user data and should be treated as a fixture, not as a real broad corpus.

3. `core-v2-heldout`
   Creates route-level held-out cases with negative shortcuts and leakage
   controls.

4. `core-v2-focus`
   Builds a route-balanced focus packet under the 15,000 fact hot proof window
   and removes exact held-out facts.

5. `core-v2-density`
   Compares packed residual memory with a linear fact baseline. It may mark a
   density candidate, but it must not prove nonlinear memory at fixture scale.

6. `core-v2-run`
   Runs a local route field from query text to an evidence-bound answer or
   refusal. It can answer only inside the local fixture contract.

7. `core-v2-pack-hot`
   Materializes a compact hot packet storage report. It checks fixed record
   size and 6 MiB budget fit, but it is not cache-only execution proof.

8. `core-v2-claim-gate`
   Aggregates all local stages and keeps hard blockers active.

## Current Boundary

The current Core V2 state is useful for engineering:

- it verifies the path from corpus to local run;
- it tests held-out removal and route-balanced focus;
- it exposes density candidate metrics honestly;
- it keeps hard claims closed.

The next real proof step is not another fixture command. It is a broad,
independent corpus path with stronger held-out, multi-profile density, and
cache-only runtime evidence.
