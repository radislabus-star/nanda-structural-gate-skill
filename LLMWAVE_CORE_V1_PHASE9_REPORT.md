# LLMWave Core V1 Phase 9 Report

Phase 9 adds the first answer verifier gate for Core V1. It does not make the
system a chatbot. It checks whether a Phase 8 surface candidate can become a
local verified answer state.

## Command

```bash
nanda-llmwave-big core-v1-answer-verifier \
  --text "Has customs cleared the goods?" \
  --format json
```

## Verdict

```text
CORE_V1_ANSWER_VERIFIER_READY_LOCAL_ONLY
```

This means a local evidence-bound answer state can be verified for the fixture.
It does not mean general chat readiness, free-form generation, nonlinear-memory
proof, or broad LLM readiness.

## Verification Rules

- the surface must cite its evidence routes;
- actor/object role bindings must survive surface generation;
- forbidden shortcuts must remain blocked;
- WATCH/split surfaces cannot become final answers;
- unsupported positive clearance claims remain rejected.

## Fixed Record

Phase 9 introduces `CoreV1AnswerVerificationRecord32`, a 32-byte verification
record:

- route id
- evidence id
- surface id
- verifier id
- state id
- flags
- evidence score
- role score
- shortcut score
- safety score
- final score
- permission score

## Fixture Decision

For:

```text
Has customs cleared the goods?
```

the verifier permits only:

```text
LOCAL_FINAL_REFUSAL
```

with text:

```text
Not proven: customs release still needs declaration/release evidence.
```

It still blocks the shortcut:

```text
invoice_or_payment_implies_customs_release
```

## Exit Criteria

- verified answer cites evidence routes;
- verified answer preserves role bindings;
- forbidden shortcut blocks positive claim;
- unsafe surfaces are rejected.

## Claim Boundary

```text
answer_verifier_v1_implemented = true
verified_refusal_ready         = true
positive_answer_ready          = false
free_form_generation           = false
feedback_learning_ready        = false
general_chat_ready             = false
llm_ready                      = false
nonlinear_memory_proven        = false
```

## Next Phase

```text
phase-10-feedback-learning-v1
```
