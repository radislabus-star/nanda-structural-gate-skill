# LLMWave Core V1 Phase 6 Report

## Phase

`phase-6-active-field-retrieval-v1`

## Command

```bash
nanda-llmwave-big core-v1-active-retrieval --text "Has customs cleared the goods?" --format json
```

## Verdict

`CORE_V1_ACTIVE_FIELD_RETRIEVAL_READY_NOT_REASONING`

This means Core V1 can use a Phase 5 query wave to select or block a route peak
through an active field pass. It does not mean schema reasoning, answer
generation, chat readiness, or nonlinear-memory proof.

## Pipeline

```text
query wave
  -> coarse route peaks
  -> local focus packet
  -> field pass
  -> peak state
```

## Required Field States

| State | Meaning |
|---|---|
| `FIELD_FOCUSED` | one coherent route can move to schema reasoning |
| `FIELD_CONTESTED` | routes compete too closely; block answer |
| `FIELD_THIN` | lexical signal exists but evidence/route support is too weak |
| `FIELD_REVERSED` | role polarity is reversed; hard stop |
| `FIELD_NOISY` | partial route signal is polluted by unrelated terms |
| `FIELD_NO_ANSWER` | no coherent route was found |

## Exit Criteria

| Criterion | State |
|---|---|
| retrieval beats lexical baseline on hard route traps | pass |
| contested fields block answer generation | pass |
| anti-wave suppression remains local | pass |
| all required field states are represented | pass |

## Claim Boundary

| Claim | State |
|---|---|
| active field retrieval v1 implemented | true |
| retrieval ready | true |
| schema reasoning ready | false |
| answer generation ready | false |
| llm ready | false |
| nonlinear memory proven | false |

## Next Phase

`phase-7-schema-reasoning-v1`

Turn focused field peaks into explicit schema answer plans with actor, action,
object, condition, evidence, time/currentness, route, and forbidden shortcut.
