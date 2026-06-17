# NANDA Structural Gate Roadmap

The goal is to turn the current placeholder into a real local verifier that an
agent can call before finalizing relation-heavy answers.

## Phase 0 - Bookmark

Status: done.

- Skill triggers on role/route/obligation/evidence-chain tasks.
- Local CLI exists.
- Agent is forbidden from claiming broad NANDA superiority.

## Phase 1 - SPARSE-TRIAD-0 Eval

Status: started with PASS, role-swap VETO, route-splice VETO, and
evidence-conflict VETO examples.

Build a benchmark eval in `/home/ubu/projects/nando-wave` or this project:

- Follow `/home/ubu/projects/nanda-structural-gate-skill/ARCHITECTURE.md`.
- Generate correct triads and corrupted role-swap triads.
- Generate route-splice cases where exact candidate facts come from different
  source groups.
- Track exact-baseline false accepts.
- Keep a code-flow template so agents can check source/runtime/CLI/plugin
  structures during implementation.
- Encode role/entity/relation as compact wave/VSA vectors.
- Score local bindings and composite triad modes.
- Compare against:
  - exact symbolic rule;
  - token overlap;
  - cosine/vector baseline;
  - simple graph consistency rule.
- Pass only if NANDA catches structural errors that cheap baselines miss.

## Phase 2 - CLI

Replace the stub with:

```bash
nanda-check --triads task.json --format text
nanda-check --triads task.json --format json
```

Required output:

- `verdict`;
- stable triads;
- weak triads;
- conflicts;
- evidence gaps;
- baseline comparison summary.

## Phase 3 - Agent Gate

Make agent usage cheap:

- accept JSON, YAML, or simple pipe table;
- provide short reports by default;
- keep full traces in local files;
- never require the agent to hand-write a giant packet.

## Phase 4 - Real Work Tests

Use domains where LLMs often confuse routes:

- customs/import chain;
- contract party responsibility;
- certification holder/manufacturer/applicant;
- CRM deal stage and document status;
- code dependency and ownership flow.

The project is useful only if it produces fewer structural mistakes than an LLM
alone and gives a clear `WATCH`/`VETO` when evidence is insufficient.
