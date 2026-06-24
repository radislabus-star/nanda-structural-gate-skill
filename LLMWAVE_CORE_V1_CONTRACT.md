# LLMWave Core V1 Contract

Status: Phase 1 contract recorded.
Created: 2026-06-25.
Machine report: `nanda-llmwave-big core-v1-contract --format json`.

## Purpose

`LLMWave Core v1` is the target model loop for the next stage of this project.
It is not another command collection and it is not a claim that a general LLM is
ready.

The intended loop is:

```text
Corpus / Atlas
  -> nonlinear memory write
  -> active field
  -> route/schema retrieval
  -> surface generation
  -> evidence / anti-wave verification
  -> feedback learning
  -> consolidation
  -> eval against baselines
```

This document defines the component boundaries and claim boundaries before
Phase 2 starts changing field execution.

## Components

| Component | Route | Responsibility | Input | Output |
|---|---|---|---|---|
| Cold Atlas | `cold-atlas` | Store large corpus indexes, evidence, and schema candidates. | public-safe corpus or project artifact | candidate focus sets and evidence refs |
| Memory Writer | `memory-write` | Write schema, surface, and residual memory without making a flat token table the primary memory. | observed facts, surfaces, feedback | schema residuals, surface families, evidence refs |
| Active Core | `active-core` | Settle a small focus packet inside the hot field budget. | compact focus records, lanes, centroids | peaks, support, anti-support, field state |
| Field Engine | `field-core` | Compute peak, coherence, verdict, anti-wave, and feedback delta. | field input records, lenses, feedback | field pass report |
| Schema Field | `l3-schema-field` | Own roles, routes, operators, evidence, and decision structure. | schema records, operator records, route context | schema answer plan, route vetoes, role expectations |
| Surface Field | `l2-surface-field` | Produce roots, morphemes, word forms, phrase, and style candidates. | surface context, schema bias, style profile | ranked surface candidates |
| Answer Generator | `answer-surface` | Materialize an evidence-bound surface from a schema answer plan. | schema answer plan, field evidence, surface memory | draft answer surface |
| Verifier | `answer-verifier` | Authorize or reject a draft answer against field evidence and anti-wave state. | draft answer, field pass, evidence refs, claim policy | PASS, WATCH, VETO, or NO_EVIDENCE |
| Feedback Memory | `feedback-memory` | Turn accept/reject/watch/correct feedback into explicit memory packets. | answer report, feedback decision, correction | positive lanes, negative lanes, schema/surface corrections |
| Consolidator | `consolidation` | Merge duplicates, promote schemas, decay noise, and preserve conflicts. | memory packets, feedback, residuals, eval metrics | compacted memory and next Atlas |
| Eval Harness | `eval-harness` | Compare memory, field, answer, and feedback behavior against baselines. | heldout suites, baselines, reports | claim gate evidence |

## Required Boundaries

1. L2 does not own L3 schema decisions.
   Owner: `Schema Field`.
   Enforced by: L2/L3 boundary tests and answer verifier.

2. L3 does not store raw UTF-8 dictionary as primary cognition.
   Owner: `Memory Writer`.
   Enforced by: memory writer density reports and surface-family reports.

3. Verifier does not generate.
   Owner: `Verifier`.
   Enforced by: answer verifier contract tests.

4. Generator does not self-authorize PASS.
   Owner: `Answer Generator`.
   Enforced by: answer surfaces must pass verifier.

5. Feedback changes memory only through explicit packets.
   Owner: `Feedback Memory`.
   Enforced by: feedback packet audit and route guard.

## Claim Boundary

Phase 1 opens only this claim:

```text
core_contract_recorded = true
claim_boundary_table_present = true
```

Phase 1 does not open:

```text
field_core_as_sole_engine = false
evidence_bound_answer_ready = false
feedback_learning_ready = false
nonlinear_memory_proven = false
llm_ready = false
```

Forbidden claims until later evals open them:

```text
LLM ready
general chatbot ready
nonlinear memory proven
cache-only execution proven
full semantic understanding proven
GPT-class comparison won
```

## Phase 1 Exit Criteria

```text
core_contract_recorded = true
claim_boundary_table_present = true
l2_l3_boundary_recorded = true
verifier_generator_boundary_recorded = true
feedback_packet_boundary_recorded = true
implementation_started = false
```

`implementation_started=false` is intentional here. Phase 1 is the contract
gate before Phase 2 field-core cutover.

## Commands

Machine JSON:

```bash
nanda-llmwave-big core-v1-contract --format json
```

Markdown:

```bash
nanda-llmwave-big core-v1-contract --format md
```

Text:

```bash
nanda-llmwave-big core-v1-contract --format text
```

## Next Phase

The next phase is:

```text
phase-2-field-core-cutover
```

Phase 2 may start only after the NANDA self-gate workflow passes for the
declared action:

```text
llmwave.field_core_cutover
```
