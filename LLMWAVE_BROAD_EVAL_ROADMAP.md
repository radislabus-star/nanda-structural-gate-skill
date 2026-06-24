# LLMWave Broad Cognition Eval Roadmap

Status: planning document.
Updated: 2026-06-24.

Implementation checkpoint:

```text
first broad eval path: implemented
commands:
  broad-corpus-build
  broad-dataset-doctor
  broad-eval-suite-build
  broad-heldout-build
  broad-focus-build
  broad-eval-run
  broad-baseline-duel
  broad-chat-loop-eval
  llmwave-readiness
current honest top state:
  BROAD_DATASET_MEDIUM
  BROAD_EVAL_GENERATION_READY_NOT_CHAT
  BROAD_CHAT_LOOP_READY_NOT_GENERAL_LLM
  LLMWAVE_READY_CANDIDATE_EXTERNAL_MEDIUM when memory proof, broad dataset
  doctor, held-out focus broad eval, baseline duel, and chat-loop evidence are
  supplied
still blocked:
  general LLM readiness
```

This roadmap is the next block after the current memory proof path:

```text
nonlinear_memory_proven = true
density_hot_artifact_ready = true
llm_ready = false
blocked_by = ["broad_llm_eval_missing"]
```

The goal is to close `broad_llm_eval_missing` honestly. This document does not
claim that LLMWave is a chatbot or a general LLM. It defines the broad eval
system needed before any `llmwave_ready_candidate` claim can open.

Current implementation separates controlled readiness from external-medium
readiness:

```text
broad-dataset-doctor
  -> broad-heldout-build
  -> broad-focus-build
  -> broad-eval-run --focus-packet
  -> llmwave-readiness --broad-dataset-doctor
```

That path can open `LLMWAVE_READY_CANDIDATE_EXTERNAL_MEDIUM`; it deliberately
does not open `llm_ready`.

The normal public-safe strong seed is
`examples/llmwave-big-broad-public-corpus-100k.txt`: 100,000 generated
structural facts across 10 domains and 50 routes. It is an artifact seed for
corpus/eval pipeline development and intentionally excludes user/private data.
The 96-fact file remains a fast smoke seed.
The 1M public-safe stress corpus is generated under ignored `.nanda/` with the
same generator and is not committed. Current local measurement: 1,000,000
facts, 224 MiB text source, 382 MiB JSON artifact, `BROAD_DATASET_STRONG`,
domain-route-balanced held-out/focus reports, 15,000 fact focus packet,
near-duplicate leakage reporting, readiness candidate gating, and
`llm_ready=false`.

## Core Question

The next proof must answer:

```text
Is LLMWave becoming a small cognitive wave model,
or is it only a set of local memory/proof fixtures?
```

The broad eval must test five axes together:

```text
1. memory
2. generation
3. structural understanding
4. context retention
5. refusal of false routes and shortcuts
```

## Maturity Ladder

```text
Level 0: storage proof
Level 1: retrieval proof
Level 2: route understanding
Level 3: answer generation
Level 4: context retention
Level 5: adversarial refusal
Level 6: learning/update
Level 7: broad mixed corpus
Level 8: chat-like loop
Level 9: LLMWave readiness candidate
```

Current state after density proof and hot packet work:

```text
storage proof: present
density proof: present
hot artifact: present
controlled nonlinear memory: present
broad generation/chat: not proven
```

## Phase 1: Broad Eval Contract

Create a contract that records what is being evaluated and which claim is
allowed to open.

Required fields:

```text
BroadEvalContract
  corpus_profile
  task_families
  answer_modes
  context_depth
  adversarial_modes
  baseline_modes
  verdict_policy
  claim_boundary
```

Top-level verdicts:

```text
BROAD_EVAL_BLOCKED
BROAD_EVAL_WEAK
BROAD_EVAL_ROUTE_READY
BROAD_EVAL_GENERATION_READY_NOT_CHAT
BROAD_EVAL_CHAT_LOOP_WEAK
LLMWAVE_READY_CANDIDATE
```

The eval must be able to block claims. A pretty answer is not enough.

## Phase 2: Task Families

The suite must include at least these task families.

### Exact Recall

The model retrieves a specific fact from evidence.

Metrics:

```text
recall_accuracy
wrong_entity_rate
not_found_rate
```

### Role Binding

The model keeps subject, relation, object, owner, and party roles separate.

Metrics:

```text
role_error_rate
subject_object_swap_rate
owner_confusion_rate
```

### Route Reasoning

The model chooses the correct route and refuses wrong route repairs.

Examples:

```text
payment route != certification route
runtime state != code bug
UI display != backend truth
```

Metrics:

```text
route_accuracy
foreign_route_pull_rate
route_splice_rejection
```

### Multi-Hop Reasoning

The model follows a chain of 2-5 steps without unsupported jumps.

Metrics:

```text
multi_hop_pass_rate
chain_break_rate
unsupported_jump_rate
```

### Context Retention

The model keeps active facts across turns and handles correction.

Metrics:

```text
context_depth_pass_rate
context_decay_rate
route_drift_rate
```

### Answer Generation

The model produces grounded answer surfaces instead of only selecting a fact.

Answer modes:

```text
short_answer
structured_answer
operator_style_answer
```

Metrics:

```text
answer_completeness
answer_grounding
unsupported_claim_rate
surface_quality_score
```

### Adversarial Shortcut Rejection

The model refuses lexical shortcuts, route traps, role swaps, stale facts, and
conflicting evidence.

Metrics:

```text
false_shortcut_rejection_rate
negative_lane_success_rate
anti_wave_effect
veto_precision
```

### Learning / Feedback

The model applies accept/reject feedback as a memory packet that changes the
next field pass.

Metrics:

```text
feedback_application_rate
repeated_error_rate
memory_delta_effect
over_suppression_rate
```

## Phase 3: Corpus Architecture

Do not treat one giant corpus as one proof. Build layered corpora:

```text
corpus/
  micro/
  medium/
  broad/
  adversarial/
  heldout/
  negative/
  dialogue/
```

Micro corpus:

```text
100-500 facts
fast
deterministic
used in normal tests
```

Medium corpus:

```text
5k-20k facts
route-balanced
domain-mixed
used for proof development
```

Broad corpus:

```text
50k-200k facts
multi-domain
used manually or nightly
```

Adversarial corpus:

```text
role swaps
same names
duplicate routes
near-root collisions
false invoices
stale facts
conflicting evidence
```

Dialogue corpus:

```text
multi-turn
stateful
feedback
correction
follow-up
```

## Phase 4: Eval Case Format

Eval cases must verify route and evidence, not only text.

Required shape:

```json
{
  "case_id": "contract-role-001",
  "family": "role_binding",
  "input": {
    "context_triads": [],
    "question": "Who authored the protocol?"
  },
  "expected": {
    "answer_contains": ["supplier"],
    "forbidden_contains": ["buyer authored protocol"],
    "required_routes": ["protocol_direction"],
    "forbidden_routes": ["buyer_original_contract"]
  },
  "negative_shortcuts": [
    {
      "shortcut": "protocol text came from buyer",
      "reason": "lexical proximity trap"
    }
  ],
  "scoring": {
    "must_ground": true,
    "allow_watch": false,
    "allow_not_proven": true
  }
}
```

## Phase 5: Model Output Contract

The model output must expose field state and claim boundaries.

Required shape:

```json
{
  "field_state": "stable",
  "selected_route": "supplier_protocol_route",
  "answer_mode": "short_answer",
  "evidence": [],
  "anti_wave_used": true,
  "suppressed_shortcuts": [],
  "surface": "Protocol authorship is supplier-side.",
  "claim_boundary": {
    "can_answer": true,
    "requires_human_review": true
  }
}
```

Without this intermediate output, a correct string may still be an accidental
guess.

## Phase 6: Baselines

Every broad eval must compare against baselines.

Required baselines:

```text
lexical baseline
vector/cosine-like baseline
route-only baseline
flat memory baseline
markov/ngram surface baseline
```

LLMWave must win where the architecture claims it should win:

```text
role swaps
foreign pull
multi-hop routes
shortcut rejection
memory density
feedback suppression
```

## Phase 7: Scoring

Memory metrics:

```text
bytes_per_useful_fact
schema_reuse_ratio
residual_saving_ratio
active_packet_bytes
hot_packet_bytes
```

Reasoning metrics:

```text
role_accuracy
route_accuracy
multi_hop_accuracy
context_retention_score
```

Safety metrics:

```text
false_positive_rate
false_shortcut_acceptance
veto_precision
watch_quality
unsupported_claim_rate
```

Generation metrics:

```text
surface_grounding_rate
required_fact_coverage
forbidden_claim_rate
answer_compression
style_match
```

Field metrics:

```text
peak_margin
coherence
foreign_pull_energy
anti_wave_delta
contested_peak_count
```

Learning metrics:

```text
feedback_delta
shortcut_suppression_persistence
accepted_memory_reuse
reverted_error_rate
```

## Phase 8: Broad Eval CLI

Planned command surface:

```bash
nanda-llmwave-big broad-corpus-build \
  --source data/llmwave/raw \
  --profile mixed \
  --out .nanda/llmwave-big/broad-corpus.json

nanda-llmwave-big broad-dataset-doctor \
  --corpus .nanda/llmwave-big/broad-corpus.json \
  --out .nanda/llmwave-big/broad-dataset-doctor.json

nanda-llmwave-big broad-eval-suite-build \
  --corpus .nanda/llmwave-big/broad-corpus.json \
  --families recall,role,route,multihop,context,generation,adversarial,feedback \
  --out .nanda/llmwave-big/broad-eval-suite.json

nanda-llmwave-big broad-heldout-build \
  --corpus .nanda/llmwave-big/broad-corpus.json \
  --out .nanda/llmwave-big/broad-heldout.json

nanda-llmwave-big broad-focus-build \
  --corpus .nanda/llmwave-big/broad-corpus.json \
  --heldout-suite .nanda/llmwave-big/broad-heldout.json \
  --out .nanda/llmwave-big/broad-focus.json

nanda-llmwave-big broad-eval-run \
  --corpus .nanda/llmwave-big/broad-corpus.json \
  --suite .nanda/llmwave-big/broad-heldout.json \
  --focus-packet .nanda/llmwave-big/broad-focus.json \
  --hot-packet .nanda/llmwave-big/density-ablation.hot \
  --format json \
  --out .nanda/llmwave-big/broad-eval-report.json

nanda-llmwave-big broad-baseline-duel \
  --eval-report .nanda/llmwave-big/broad-eval-report.json \
  --baselines lexical,flat,route-only,markov \
  --format json

nanda-llmwave-big llmwave-readiness \
  --memory-final-proof .nanda/llmwave-big/memory-final-proof.json \
  --broad-dataset-doctor .nanda/llmwave-big/broad-dataset-doctor.json \
  --broad-eval .nanda/llmwave-big/broad-eval-report.json \
  --baseline-duel .nanda/llmwave-big/broad-baseline-duel.json \
  --format json
```

## Phase 9: Answer Levels

Separate answer capability levels:

```text
Level A: evidence-bound answer
Level B: route-bound answer
Level C: generative explanation
Level D: open chat
```

Near-term target:

```text
Level B + partial Level C
```

Do not claim open chat until the dialogue eval supports it.

## Phase 10: Generation Pipeline

The generation path should be field-driven:

```text
Question Wave
  -> Route Peak
  -> Evidence Bind
  -> Answer Frame
  -> L2 Word Field / Surface Bank
  -> Anti-Wave Claim Filter
  -> Output
```

The model must first decide:

```text
can answer?
what is proven?
which route?
which answer mode?
which claims are forbidden?
```

Then it may produce text.

## Phase 11: L2 Word Field

L2 should hold:

```text
surface families
roots
morphemes
suffixes
operator phrases
style templates
short answer forms
```

It should not devolve into a plain `token_id -> UTF-8 string` table. The target
shape is:

```text
meaning role -> surface family -> materialized phrase
```

## Phase 12: L3 Schema Field

L3 should hold:

```text
routes
roles
operators
schema memory
cause/effect
permission/forbidden
evidence binding
anti-wave lanes
```

L3 chooses the meaning and route. L2 materializes the answer surface.

## Phase 13: Anti-Wave Broad Eval

Anti-wave must prove rescue without over-suppression.

Eval traps:

```text
same entity, different role
same invoice, different shipment
same file, different route
same tail, different namespace
same word, different legal effect
```

Metrics:

```text
anti_wave_rescue_rate
anti_wave_overkill_rate
```

## Phase 14: Session Field

The model needs a session field for multi-turn context:

```text
accepted facts
rejected shortcuts
current route
active entities
topic drift
last answer commitments
```

Fresh evidence and explicit correction must outrank stale assumptions.

## Phase 15: Feedback As Training

Feedback must become an applicable memory packet:

```json
{
  "feedback_type": "reject_shortcut",
  "suppressed_route": "wrong_route",
  "reinforced_route": "correct_route",
  "scope": "local/session/profile",
  "decay": 0.8
}
```

The next field pass must change measurably.

## Phase 16: Big Mixed Stress

Stress domains:

```text
Rust project architecture
contracts
EDI/EDO
customs/logistics
finance/news
synthetic route traps
dialogue memory
```

The target failure mode is cross-domain role and route confusion.

## Phase 17: Reports

Machine report:

```json
{
  "safe_to_claim_llm": false,
  "safe_to_claim_nonlinear_memory": true,
  "failed_families": [],
  "route_errors": [],
  "unsupported_claims": []
}
```

Human report:

```text
Where the model is strong
Where the model is weak
Which claims are still blocked
Which repair should be next
```

## Phase 18: Final Claim Boundary

After broad eval, final proof should track separate claims:

```json
{
  "nonlinear_memory_proven": true,
  "field_reasoning_ready": false,
  "answer_generation_ready": false,
  "chat_loop_ready": false,
  "llmwave_ready_candidate": false,
  "llm_ready": false
}
```

The target is not to flip `llm_ready` prematurely. The target is to expose which
cognitive functions are proven.

## Phase 19: First Winning Milestones

First milestone:

```text
BROAD_EVAL_ROUTE_READY
```

Suggested thresholds:

```text
role_accuracy >= 0.90
route_accuracy >= 0.90
false_shortcut_rejection >= 0.90
unsupported_claim_rate <= 0.05
baseline win on route/adversarial tasks
```

Second milestone:

```text
BROAD_EVAL_GENERATION_READY_NOT_CHAT
```

Suggested thresholds:

```text
surface_grounding >= 0.85
forbidden_claim_rate <= 0.05
answer_completeness >= 0.80
```

Third milestone:

```text
LLMWAVE_READY_CANDIDATE
```

Suggested thresholds:

```text
context retention >= 0.80 across N turns
feedback correction works
mixed corpus stable
broad baseline duel won in target tasks
```

## Phase 20: Implementation Order

Recommended order:

```text
1. broad_eval_contract
2. broad_eval_case format
3. micro suite: recall/role/route/adversarial
4. broad_eval_run over current field
5. baseline lexical/flat comparison
6. answer output contract
7. surface generation eval
8. context retention eval
9. feedback eval
10. final llmwave_readiness gate
```

Do not start from a huge corpus. Start from a correct eval mechanism, then
scale the corpus.

## End State

The target architecture after this block:

```text
Cold Corpus / Atlas
        |
        v
Focus Builder
        |
        v
L3 Schema Field
        |
        +--> Anti-Wave Safety
        |
        +--> Route/Role Reasoning
        |
        v
L2 Word Field
        |
        v
Answer Surface
        |
        v
Broad Eval
        |
        v
Claim Boundary
```

The model should not be judged by the question "can it chat?" first. It should
be judged by proven cognitive functions:

```text
memory proof
route proof
anti-wave proof
grounded answer proof
context proof
feedback proof
baseline duel proof
```

Only after these pass can `llmwave_ready_candidate` become a serious claim.
