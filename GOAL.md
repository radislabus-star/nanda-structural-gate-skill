# GOAL: NANDA Structural Gate Skill

## Objective

Create a local Codex skill and CLI gate that force structural relation checking
before final answers on role/route/evidence-heavy tasks after a measurable
complexity threshold is crossed.

The end state is not a chatbot. The end state is:

```text
LLM extracts triads -> nanda-check verifies structural binding -> agent answers with PASS/WATCH/VETO awareness
```

## Why This Exists

LLMs often handle individual facts correctly but confuse the combined relation:

- who sold to whom;
- who owns the document;
- which route belongs to payment vs delivery;
- whether facts from two different routes were spliced into one answer;
- whether one evidence reference was bound to incompatible facts;
- who is responsible for certification;
- which evidence supports which claim.

NANDA should test whether wave/VSA-style composite modes can catch broken
bindings cheaply enough to become a mandatory local gate.

The gate should not run for every trivial statement. It should run when the
agent has enough variables and links that role confusion becomes likely.

## Definition Of Done

- Runtime skill exists in `.codex/skills/nanda-structural-gate`.
- Source skill exists in this project.
- `nanda-check` exists as a callable V0 CLI.
- Checker input/output architecture is fixed before implementation.
- Triad packet contract is documented.
- Complexity threshold is documented and used by the agent before calling the
  gate.
- `SPARSE-TRIAD-0` examples prove the CLI can return PASS and VETO.
- A 150-case synthetic benchmark proves the current role-swap and route-splice
  gates are wired.
- Route-splice benchmark records exact-baseline false accepts.
- Baselines are included before any claim of novelty.
- Agent workflow says `WATCH` when the checker is missing or inconclusive.

## Milestones

1. Bookmark and stub
   - Status: done.

2. Sparse triad eval
   - Use the V0 architecture in `ARCHITECTURE.md`.
   - Build synthetic role-swap and route-conflict cases.
   - Include low-complexity cases that should not require the gate.
   - Include high-complexity cases where the gate is mandatory.
   - Compare against exact rule, token overlap, cosine, and graph rule.

3. Real CLI
   - Accept triad JSON/YAML/text.
   - Return text and JSON verdicts.
   - Save optional trace files locally.

4. Skill hardening
   - Keep `SKILL.md` compact.
   - Move details into references.
   - Validate with local Rust smoke tests.

5. Real domain trials
   - Customs chain.
   - Contract responsibility.
   - Certification holder/applicant.
   - CRM deal state.
   - Code dependency flow.

## Non-Goals

- Do not claim NANDA understands text by itself.
- Do not replace RAG, graph search, source verification, or LLM reasoning.
- Do not use phase/coherence as proof of truth.
- Do not make a sales story before the eval beats baselines.

## Completed State

This project is complete as a first public skill repository when:

- `scripts/test-local.sh` passes;
- the runtime skill installs with `scripts/install-local.sh`;
- `nanda-check` returns `PASS` for the clean example;
- `nanda-check` returns `VETO` for the role-swap example;
- `scripts/benchmark-v0.sh` passes 50 clean, 50 swapped, and 50 route-splice
  synthetic cases.
