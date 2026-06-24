# LLMWave Core V1 Execution Plan

Status: planning document.
Created: 2026-06-24.
Scope: record only. Do not execute this plan unless the user explicitly starts a
GOAL for it.

## Purpose

This plan defines the next strict path from the current NANDA / LLMWave
prototype toward a real `LLMWave Core v1`.

The goal is not to add more decorative commands. The goal is to assemble one
complete model loop:

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

The final target is:

```text
LLMWave Core v1:
  a small wave-memory model that loads a large corpus, writes compact reusable
  memory, builds an active field, reads a query as a wave, selects route/schema
  structure, generates an evidence-bound answer, verifies it with anti-wave and
  evidence lenses, learns from feedback, consolidates memory, and beats simple
  lexical/RAG/graph baselines on structural tasks.
```

## Current Honest State

Already present:

```text
NANDA Structural Gate:
  usable structural verifier and Codex failure firewall.

Unified field:
  started. field_core contains common peak/coherence/verdict/anti-wave/readout
  operations, but it is not yet the sole engine for every route.

NANDA-6M / packed runtime:
  cache-resident packed runtime candidate, 6 MiB budget, packed lanes, focus
  windows, anti-wave/replay lanes, and density benches.

LLMWave Big:
  many component fixtures exist: L2/L3 coupling, HRR/schema binding, query wave,
  surface generation, evidence proof, feedback memory, consolidation, density
  gates, and broad eval scaffolding.

Broad public-safe corpus path:
  1,000,000 generated facts, 10 domains, 50 routes, 1024 held-out cases,
  domain-route balanced 15,000 fact focus packet, near-duplicate leakage 0,
  LLMWAVE_READY_CANDIDATE_EXTERNAL_STRONG, llm_ready=false.
```

Not proven:

```text
llm_ready = false
nonlinear_memory_proven = false unless a future scale proof opens it
cache_only_execution_proven = false
field_core_as_sole_engine = not fully complete
general chat model = not present
```

## Claim Discipline

Never claim these until an eval/report explicitly opens them:

```text
LLM ready
general chatbot ready
nonlinear memory proven
cache-only execution proven
full semantic understanding proven
GPT-class comparison won
```

Allowed intermediate claims:

```text
structural verifier ready
field engine candidate
evidence-bound answer candidate
external-strong readiness candidate
scale-amortized density observed
nonlinear memory candidate
small-domain LLMWave candidate
```

## Execution Rules

When this plan is executed later:

1. Move sequentially by phases.
2. Keep code, tests, docs, and commits per major phase.
3. Do not push without explicit user command.
4. Do not run Windows builds without explicit user command.
5. Exclude user-private data from training/eval artifacts.
6. Do not hide failure by weakening gates.
7. Treat WATCH as unresolved.
8. Keep hot-core loops free from JSON/string/heap/hashmap unless explicitly in
   cold-path code.
9. Keep L2 and L3 separated:
   - L2 = surface, word, root, morpheme, phrase, style.
   - L3 = schema, role, route, operator, evidence, decision structure.
10. Before risky repository refactors, use the NANDA route field / dogfood
    workflow.
11. Every implementation phase must be guarded by this skill itself. Do not rely
    on memory or intuition when changing this project.

## Mandatory NANDA Self-Gate Workflow

Every phase in this plan must run through `nanda-structural-gate` as a project
guardrail. The skill is not optional for implementation phases.

### Before A Phase

Run a repository route check:

```bash
nanda dogfood . --refactor-plan --boundary-economics --format json
nanda build-atlas . --out .nanda/route-atlas.json
```

Then define a precise `action_id` for the phase, for example:

```text
llmwave.core_contract_v1
llmwave.field_core_cutover
llmwave.memory_writer_v1
llmwave.query_wave_v1
llmwave.active_field_retrieval_v1
llmwave.schema_reasoning_v1
llmwave.surface_generation_v1
llmwave.answer_verifier_v1
llmwave.feedback_learning_v1
llmwave.consolidation_v1
llmwave.nonlinear_memory_proof_v1
llmwave.broad_eval_harness_v1
```

Before editing, run an action guard:

```bash
nanda guard-action .nanda/route-atlas.json \
  --action-id <action_id> \
  --boundary-economics \
  --format json
```

If the result is `HARD_STOP`, `VETO`, `WATCH`,
`ANALYSIS_INSUFFICIENT`, or `safe_to_edit=false`, do not edit yet. Repair the
route/action/evidence plan first.

### During A Phase

Keep changes inside the declared action route. If a phase needs to cross a
boundary, declare a shared contract action instead of silently widening the
diff.

Examples:

```text
shared.field_core_contract
shared.memory_feedback_contract
shared.answer_verifier_contract
shared.version_bump_contract
```

Do not mix unrelated routes such as:

```text
field_core cutover + broad corpus generator
memory writer + contract gate
surface generator + structural gate firewall
answer verifier + Windows installer
```

### After A Diff

Before tests or commit, save the diff and guard it:

```bash
git diff > /tmp/llmwave-phase.diff
nanda guard-diff .nanda/route-atlas.json \
  --action-id <action_id> \
  --diff /tmp/llmwave-phase.diff \
  --boundary-economics \
  --format json
```

If the diff crosses forbidden routes, returns `VETO`, returns `WATCH`, or says
`safe_to_edit=false`, repair the diff before continuing.

### Before Commit

Run the route field again:

```bash
nanda dogfood . --refactor-plan --boundary-economics --format json
```

The phase is not committable unless:

```text
the action route is clear
foreign_pull is empty or explicitly repaired
mixed_candidate_groups is empty or explicitly split
boundary decision is not VETO
claim boundaries remain honest
```

### After A Phase

Every phase completion note must record:

```text
action_id
routes touched
routes forbidden and avoided
NANDA pre-check verdict
NANDA diff-check verdict
NANDA post-check verdict
tests run
claim gates opened
claim gates still blocked
```

This rule exists because LLMWave development can otherwise drift back into
"more commands" instead of the core loop. The skill must protect the plan from
route creep, fake proof, and accidental claim inflation.

## Phase 1: Core V1 Contract

Objective:

Create a single architectural contract that defines what `LLMWave Core v1`
is and is not.

Deliverables:

```text
LLMWAVE_CORE_V1_CONTRACT.md
llmwave_core_v1_contract.json or equivalent report
```

Define components:

```text
Cold Atlas
Memory Writer
Active Core
Field Engine
Schema Field
Surface Field
Answer Generator
Verifier
Feedback Memory
Consolidator
Eval Harness
```

Required boundaries:

```text
L2 does not own L3 schema decisions.
L3 does not store raw UTF-8 dictionary as primary cognition.
Verifier does not generate.
Generator does not self-authorize PASS.
Feedback changes memory only through explicit packets.
```

Exit criteria:

```text
core_contract_recorded = true
claim_boundary_table_present = true
```

## Phase 2: Field Core Cutover

Objective:

Make `field_core` the shared physics layer for structural, packed, and
LLMWave routes.

Work:

1. Inventory all duplicated field operations:
   - peak detection;
   - coherence;
   - route score;
   - anti-wave suppression;
   - verdict;
   - field state;
   - centroid drift;
   - support / anti-support.
2. Move common operations into `field_core`.
3. Keep old command surfaces as compatibility wrappers.
4. Add common data types:

```text
FieldInput
FieldRecord
FieldLens
FieldPass
FieldPeak
FieldVerdict
FieldSuppression
FieldFeedbackDelta
```

Exit criteria:

```text
structural search can run through field_core
packed route can run through field_core
LLMWave route can run through field_core
field equivalence tests pass
field_core_as_sole_engine_candidate = true
```

Do not claim final sole-engine status until compatibility wrappers no longer
contain independent scoring physics.

## Phase 3: Memory Writer V1

Objective:

Replace naive raw token storage as the primary model memory with reusable
schema/surface/residual memory.

Memory levels:

```text
Observed Surface:
  exact forms seen in corpus.

Surface Family:
  roots, stems, suffix families, phrase templates, style variants.

Schema / Relation Residual:
  schema id, role bindings, relation phase, polarity, residual delta, evidence.
```

Work:

1. Define memory record format for schema residuals.
2. Define surface family references.
3. Define evidence pointer/hash fields.
4. Write reports:

```text
raw_observed_bytes
schema_reuse_bytes
residual_bytes
surface_family_bytes
rejected_duplicate_count
rejected_noise_count
```

Exit criteria:

```text
residual_write_path_active = true
raw_dictionary_is_not_primary_memory = true
memory_write_report_present = true
```

## Phase 4: Nonlinear Memory Proof V1

Objective:

Prove or block nonlinear memory honestly.

Definition:

Nonlinear memory means that new useful facts increasingly reuse existing
schema/surface/field structure, so bytes per useful fact fall without degrading
held-out reasoning quality.

Required metrics:

```text
bytes_per_useful_fact
schema_reuse_ratio
residual_saving_ratio
surface_family_reuse_ratio
heldout_pass_rate
role_error_rate
false_positive_rate
near_duplicate_leakage_rate
baseline_delta
```

Scale ladder:

```text
10k facts
50k facts
100k facts
250k facts
500k facts
1M facts
```

Baselines:

```text
linear full fact store
lexical retrieval
simple graph lookup
simple vector/RAG-style retrieval where available
```

Verdicts:

```text
NONLINEAR_BLOCKED
DENSITY_CANDIDATE
SCALE_AMORTIZED_DENSITY_OBSERVED
NONLINEAR_MEMORY_CANDIDATE
NONLINEAR_MEMORY_PROVEN
```

`NONLINEAR_MEMORY_PROVEN` may open only if:

```text
bytes_per_useful_fact falls across at least 3 scale points
heldout_pass_rate does not degrade materially
role_error_rate does not rise materially
false_positive_rate does not rise materially
near_duplicate_leakage remains low
LLMWave beats the selected baselines on structural tasks
```

## Phase 5: Query Wave V1

Objective:

Convert user text into a structured wave query instead of a keyword bag.

Query components:

```text
surface terms
roles
operators
negation
time/currentness
evidence demand
route hints
uncertainty
polarity
```

Question families:

```text
recall
who / what role
route decision
contradiction
missing evidence
multi-hop
generate explanation
refuse unsupported
```

Exit criteria:

```text
same-meaning paraphrase selects same route peak
role-swap query triggers reversed polarity or VETO
missing-evidence query does not produce confident answer
```

## Phase 6: Active Field Retrieval V1

Objective:

Use the query wave to select coherent routes and reject lexical traps.

Pipeline:

```text
query wave
  -> coarse route peaks
  -> local focus packet
  -> field pass
  -> peak state
```

Required field states:

```text
FIELD_FOCUSED
FIELD_CONTESTED
FIELD_THIN
FIELD_REVERSED
FIELD_NOISY
FIELD_NO_ANSWER
```

Output contract:

```json
{
  "field_state": "FIELD_FOCUSED",
  "top_peak": "...",
  "runner_up": "...",
  "peak_margin": 0.0,
  "coherence": 0.0,
  "anti_wave_hits": [],
  "safe_to_answer": true
}
```

Exit criteria:

```text
retrieval beats lexical baseline on hard route traps
contested fields block answer generation
anti-wave suppression remains local
```

## Phase 7: Schema Reasoning V1

Objective:

Turn field peaks into reasoning structures, not just retrieved facts.

Schema answer plan:

```text
actor
action
object
condition
evidence
time/currentness
route
forbidden shortcut
```

Operators:

```text
requires
blocks
allows
contradicts
depends_on
overrides
causes
routes_to
must_not_merge
```

Required examples:

```text
A requires B
B depends_on C
C missing
=> answer says C is missing, not A is ready
```

Exit criteria:

```text
multi-hop held-out eval passes
contradiction eval refuses unsupported answer
role swap eval blocks wrong binding
```

## Phase 8: Surface Generation V1

Objective:

Generate evidence-bound answers without pretending to be a general chatbot.

Inputs:

```text
SchemaAnswerPlan
FieldEvidence
SurfaceMemory
StyleProfile
```

Allowed answer modes:

```text
short answer
explanation
reason list
missing evidence refusal
WATCH / split required
```

Forbidden generator behavior:

```text
invent facts
change roles
smooth VETO into PASS
turn WATCH into confidence
self-authorize without verifier
```

Exit criteria:

```text
answer cites evidence routes
answer refuses when field is unsafe
answer keeps role bindings
answer passes style/evidence eval
```

## Phase 9: Answer Verifier V1

Objective:

Verify generated surfaces against the field before returning them.

Checks:

```text
role consistency
route consistency
evidence coverage
anti-wave shortcut hit
missing evidence
stale evidence
surface/schema mismatch
```

Verdicts:

```text
ANSWER_PASS
ANSWER_WATCH
ANSWER_VETO
ANSWER_NO_EVIDENCE
```

Exit criteria:

```text
unsafe field cannot produce final answer
unsupported answer is rejected
evidence-bound answer can pass
```

## Phase 10: Feedback Learning V1

Objective:

Make feedback change the next field pass.

Feedback decisions:

```text
accept
reject
watch
correct
prefer_style
wrong_route
missing_evidence
role_swap
```

Memory effects:

```text
positive lane
negative lane
schema reinforcement
surface correction
anti-wave shortcut
evidence correction
```

Required eval:

```text
same query before feedback -> wrong/thin/contested
same query after feedback -> corrected/suppressed/reinforced
unrelated route remains stable
```

Exit criteria:

```text
feedback_changes_next_field_pass = true
feedback_does_not_overfit_unrelated_route = true
```

## Phase 11: Consolidation / Sleep Pass V1

Objective:

Turn accumulated memory and feedback into compact reusable structure.

Work:

1. Merge duplicates.
2. Promote repeated schemas.
3. Weaken one-off noise.
4. Preserve conflicts.
5. Preserve negative lanes.
6. Rebuild compact memory.

Metrics:

```text
memory_before
memory_after
schema_reuse_delta
heldout_delta
false_positive_delta
role_error_delta
```

Exit criteria:

```text
memory shrinks or stabilizes
heldout quality does not degrade
negative shortcuts remain suppressed
```

## Phase 12: Broad Eval Harness V1

Objective:

Create a broad eval that measures actual model behavior, not command count.

Task families:

```text
recall
role binding
route reasoning
contradiction
missing evidence
multi-hop
surface generation
refusal
feedback learning
consolidation
paraphrase stability
lexical trap
stale fact
near duplicate
noisy corpus
```

Each case must define:

```text
query
expected route
expected schema
forbidden answer
required evidence
baseline result
LLMWave result
verdict
```

Exit criteria:

```text
LLMWave beats baseline on structural tasks
open language ability remains separately unclaimed
```

## Phase 13: Corpus Scale Suite

Objective:

Use the public-safe corpus ladder to measure memory and reasoning at scale.

Scale points:

```text
10k
50k
100k
250k
500k
1M
```

For each scale point:

```text
dataset doctor
heldout build
focus build
memory write
field retrieval
answer generation
feedback eval
density eval
baseline duel
```

Required reports:

```text
scale_report.json
scale_report.md
```

Required table columns:

```text
facts
bytes
bytes_per_useful_fact
heldout_pass_rate
false_positive_rate
role_error_rate
latency
focus_size
schema_reuse_ratio
residual_saving_ratio
surface_family_reuse_ratio
```

Exit criteria:

```text
there is an honest curve, not a single impressive run
```

## Phase 14: L2 Word Field V1

Objective:

Make surface memory productive, not a token string lookup.

Store:

```text
roots
stems
suffix families
phrase fragments
observed forms
production rules
style variants
```

Avoid:

```text
primary token_id -> UTF-8 memory
100k wordforms as the main cognition layer
```

Required Russian/business surface support:

```text
case-like ending families
business wording templates
role-safe phrase choices
surface refusal phrases
```

Exit criteria:

```text
surface generation uses family/reconstruction path
raw lookup is fallback, not primary memory
```

## Phase 15: L3 Schema Field V1

Objective:

Make L3 the main cognitive field for routes, operators, roles, and evidence.

Store:

```text
operators
roles
routes
evidence types
decision owners
forbidden shortcuts
multi-hop schemas
conflict schemas
time/currentness
```

Work:

1. Induce repeated schemas.
2. Assign schema ids.
3. Rewrite facts as residuals to schemas.
4. Add schema competition.
5. Allow feedback to strengthen/weaken schemas.

Exit criteria:

```text
L3 can answer structural questions without relying on surface keyword match
```

## Phase 16: Full Answer Loop V1

Objective:

Assemble a single evidence-bound answer path.

Command shape:

```bash
nanda-llmwave-big answer-core \
  --memory .nanda/llmwave-core/memory.json \
  --query "..." \
  --format json
```

Pipeline:

```text
query text
  -> query wave
  -> active field retrieval
  -> schema reasoning
  -> evidence proof
  -> surface generation
  -> answer verifier
  -> output
```

Output contract:

```json
{
  "answer": "...",
  "field_state": "FIELD_FOCUSED",
  "safe_to_answer": true,
  "evidence": [],
  "schema": {},
  "claim_boundary": {
    "llm_ready": false,
    "evidence_bound_answer": true
  }
}
```

Exit criteria:

```text
the model can answer questions over its corpus in evidence-bound mode
```

## Phase 17: Interactive Learning Loop V1

Objective:

Allow the model to learn from user/agent feedback without manually editing the
corpus.

Command shape:

```bash
nanda-llmwave-big feedback \
  --answer answer.json \
  --decision reject \
  --correction "..."
```

Required test:

```text
answer query
reject/correct
answer same query again
verify changed field/answer
verify unrelated route is stable
```

Exit criteria:

```text
model learns from feedback packets
next pass changes for the targeted route
unrelated routes do not drift
```

## Phase 18: Performance / Cache Budget

Objective:

Measure the active path as an engineered runtime, not only an eval artifact.

Metrics:

```text
query latency
field pass latency
focus build latency
memory write latency
answer generation latency
RSS
hot core bytes
active focus bytes
cache-miss proxy where available
```

Modes:

```text
small
medium
100k atlas
1M atlas
15k focus
6M active core
```

Exit criteria:

```text
hot path avoids JSON/string/heap/hashmap inner loops
cold path may remain heavier
```

## Phase 19: Product Boundary

Objective:

Separate what is usable from what is research.

Products:

```text
NANDA Structural Gate:
  structural firewall and relation verifier.

LLMWave Core:
  experimental wave memory/reasoning engine.

LLMWave Chat:
  future evidence-bound chat surface, not yet a general chatbot.
```

Exit criteria:

```text
README and command docs clearly separate gate, core, and chat claims
```

## Phase 20: Final Claim Gates

Objective:

Create explicit gates for final claims.

Claim gates:

```text
field_core_as_sole_engine
nonlinear_memory_candidate
nonlinear_memory_proven
small_domain_llmwave
evidence_bound_chat_ready
llm_ready
```

Each gate must define:

```text
required evals
required metrics
required baselines
required negative controls
allowed safe claim
blocked claim
```

`llm_ready=true` is allowed only if:

```text
broad answer eval is strong
feedback learning works
surface generation is broad
memory density does not regress
false positive rate is low
role error rate is low
baselines are beaten
no hard claim blockers remain
```

Until then:

```text
llm_ready=false
```

## First Execution Block When User Starts GOAL

The first implementation block should be a vertical slice, not another command
collection.

Vertical slice:

```text
1M public-safe corpus
  -> memory write
  -> active focus
  -> query wave
  -> field retrieval
  -> schema answer plan
  -> surface answer
  -> verifier
  -> feedback
  -> second pass improved
```

Minimal initial scope:

```text
one public-safe domain
several competing routes
hard role/route traps
evidence-bound answer
feedback changes next pass
```

Only after the vertical slice works should the plan expand to all 10 domains.

## Completion Definition

This plan is complete only when:

```text
memory changes after feedback
field changes after memory update
answer changes after field update
quality improves measurably
baseline is beaten on structural tasks
claim boundary remains honest
```

The intended end state is not "more reports". The intended end state is a
measurable, evidence-bound, learnable wave-memory model core.
