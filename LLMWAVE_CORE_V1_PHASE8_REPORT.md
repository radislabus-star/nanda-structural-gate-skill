# LLMWave Core V1 Phase 8 Report

Phase 8 adds the first evidence-bound surface generation gate for Core V1.
It does not create a free-form chatbot. It turns a focused schema answer plan
into a constrained surface candidate that still requires Phase 9 verification
before it can become a final answer.

## Command

```bash
nanda-llmwave-big core-v1-surface-generation \
  --text "Has customs cleared the goods?" \
  --format json
```

## Verdict

```text
CORE_V1_SURFACE_GENERATION_READY_NOT_VERIFIED
```

This means the surface candidate is structurally materialized and ready for
the answer verifier. It does not mean final answer readiness, general LLM
readiness, or nonlinear-memory proof.

## Inputs

- `SchemaAnswerPlan`
- `FieldEvidence`
- `SurfaceMemory`
- `StyleProfile`

In this phase the implementation consumes the Phase 7 schema answer plan and
materializes only evidence-bound surface forms.

## Allowed Answer Modes

- short answer
- explanation
- reason list
- missing evidence refusal
- WATCH / split required

## Forbidden Behavior

- invent facts
- change roles
- smooth VETO into PASS
- turn WATCH into confidence
- self-authorize without verifier

## Fixed Record

Phase 8 introduces `CoreV1SurfaceCandidateRecord32`, a 32-byte surface
candidate record:

- route id
- evidence id
- template id
- mode id
- state id
- flags
- schema score
- evidence score
- style score
- anti score
- final score
- permission score

The record is a compact surface candidate, not a token dictionary entry and not
a final answer object.

## Fixture Surface

For:

```text
Has customs cleared the goods?
```

the surface is intentionally a missing-evidence refusal:

```text
Not proven: customs release still needs declaration/release evidence.
```

The candidate cites the route and missing-evidence dependency from the schema
answer plan and preserves the forbidden shortcut binding so the next verifier
can still reject unsupported certainty.

## Exit Criteria

- answer cites evidence routes
- answer refuses when field is unsafe
- answer keeps role bindings
- answer passes style/evidence eval

## Claim Boundary

```text
surface_generation_v1_implemented = true
evidence_bound_surface_ready      = true
free_form_generation              = false
answer_verifier_ready             = false
final_answer_ready                = false
llm_ready                         = false
nonlinear_memory_proven           = false
```

## Next Phase

```text
phase-9-answer-verifier-v1
```
