# Field Core Sole Engine Contract

Goal: all large field pipelines must use one shared field physics owner instead
of local copies of similar peak, lens, anti-wave, coherence, verdict, and
readout logic.

## Contract

`field_core` owns the shared physics:

- `FieldVector1024`
- `FieldRecord`
- `FieldPassInput`
- `FieldPassReport`
- `apply_lens_chain`
- `apply_anti_wave`
- `detect_field_peak`
- `summarize_field_coherence`
- `FieldRuntimeDualRun`
- `FieldEngineDecision`

Domain modules may still extract domain records, format reports, enforce claim
boundaries, and run hot mechanical scans. They must not become separate owners
of field physics.

## Registered Big Pipelines

`nanda-field-audit --format json` emits `sole_engine_contract`.

The required consumers are:

- `structural-search`
- `packed-hot-runtime`
- `llmwave-big-cognitive`
- `pattern16-structural-capacity`
- `llmwave-lens-scan`
- `llmwave-mature-anti-wave`
- `memory-feedback`
- `skill-repository-guard`

Each required consumer must expose evidence that it reaches `field_core` through
`FieldPassInput`, `FieldPassReport`, `FieldRuntimeDualRun`, or a shared field
engine policy. Local copies of field physics are not allowed.

## Acceptance

The sole-engine claim is allowed only when:

- all required big pipelines are registered;
- every required pipeline is field-core backed;
- every required pipeline uses the shared lens and anti-wave contract;
- no required pipeline allows local field physics copies;
- the structural cutover suite passes;
- claim boundaries keep LLM, nonlinear memory, and hardware cache proof closed.

## Agent Rule

Before a large code change:

```bash
nanda dogfood . --refactor-plan --boundary-economics --format json
nanda map-code <target> --format json
```

Before using the unified field claim:

```bash
nanda-field-audit --format json
nanda-llmwave-big claim-gate --claim field-core-sole-engine --format json
```

If `sole_engine_contract.field_core_as_sole_engine=false`, do not claim one
field engine. If `llm_ready=false` or `nonlinear_memory_proven=false`, do not
claim LLM readiness or nonlinear memory proof.

## Non-Claims

This contract does not prove:

- general LLM/chat readiness;
- global nonlinear memory;
- hardware cache residency;
- real broad-corpus cognition;
- legal, business, or security truth.

It proves only that registered large field pipelines route their shared field
physics through one engine.
