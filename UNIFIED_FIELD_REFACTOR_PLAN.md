# Unified Field Refactor Plan

Status: implemented as unified field projection, shared compute primitives,
first shared `FieldPass`, and local feedback-to-memory-delta replay.

This document prepares the migration from several field-like implementations to
one shared field contract. It is not permission to rewrite the runtime in one
large diff. The first rule is:

```text
one field contract, many domain projections
```

## Why

The project currently has three field families:

1. Structural Gate field
   - files: `src/map_gate.rs`, `src/search.rs`, `src/proof.rs`,
     `src/commands/guard.rs`;
   - purpose: route, owner, foreign pull, evidence, PASS/WATCH/VETO;
   - current shape: dynamic JSON/triad-oriented field.

2. Packed 6M field
   - files: `src/nanda_6m.rs`, `src/nanda_6m/wave.rs`,
     `src/nanda_6m/replay.rs`, `src/pack6m.rs`, `src/bench6m.rs`;
   - purpose: cache-budgeted packed records, centroids, lanes, hot-cycle;
   - current shape: fixed-size numeric records.

3. LLMWave Big field
   - files: `src/llmwave_big/*`;
   - purpose: L2 word/surface field, L3 schema field, query/lens/feedback,
     memory growth, learning eval;
   - current shape: experimental cognitive field modules.

The risk is that these three families keep inventing separate meanings for
wave, field, lens, peak, anti-wave, coherence, and memory. The refactor goal is
to preserve their different constraints while giving them one shared physics
vocabulary and one compatibility surface.

## Non-Goals

- Do not merge all field code into one huge module.
- Do not replace the packed hot core with JSON.
- Do not put heap/string/hashmap logic into packed inner loops.
- Do not rewrite `src/pattern_store.rs` as a dumping ground.
- Do not claim LLM/chat/nonlinear-memory proof from this refactor.
- Do not hardcode project-specific names, package names, routes, or examples
  into field core.
- Do not split large files only because they are large. Use evidence.

## Target Module Layout

```text
src/field_core/
  mod.rs
  basis.rs          # dimensions, axis ids, quantization policy
  vector.rs         # owned/borrowed wave vectors and dot/cosine helpers
  record.rs         # generic field record traits and ids
  peak.rs           # peak, margin, state, safe-to-answer contract
  lens.rs           # lens traits: route, role, polarity, evidence, temporal
  anti_wave.rs      # suppression lane contract
  coherence.rs      # energy, coherence, pull, field state
  feedback.rs       # accept/reject/watch replay contract
  adapters.rs       # explicit adapter traits, no domain logic

src/field_adapters/
  structural.rs     # triads/routes/owners -> field_core
  packed6m.rs       # PackedTriad32/PackedWave1024 -> field_core contract
  llmwave_big.rs    # L2/L3/schema/surface projections -> field_core
```

`field_core` must be small, stable, and domain-neutral. Domain-specific concepts
belong in adapters.

## Core Contract

Every implementation that calls itself a field should be expressible through
these concepts:

```text
FieldBasis
  dimension
  axis policy
  quantization policy
  memory budget class

FieldRecord
  id
  route/group/schema/surface axes as optional typed ids
  polarity
  confidence
  source/evidence weight
  vector/projection reference

FieldQuery
  vector
  requested axes
  lens stack
  anti-wave stack

FieldPeak
  target id
  score
  margin
  coherence
  support count
  anti-support count
  state
  safe_to_answer

FieldLens
  transforms query or scoring view
  must be explainable
  must expose what it suppresses or amplifies

AntiWave
  suppresses a specific false reading shape
  never deletes source memory
  must report suppression target and penalty/energy

FeedbackMemory
  accept/reject/watch decision
  replayable into next field pass
  local unless explicitly promoted
```

## Compatibility Rules

Structural Gate may use JSON and strings outside hot loops. Packed 6M may not.

```text
Structural adapter:
  allowed: JSON, strings, HashMap, rich explanations
  forbidden: hidden PASS without evidence

Packed adapter:
  allowed: fixed records, arrays, numeric ids, no heap in inner loop
  forbidden: JSON/string parsing in hot scan

LLMWave adapter:
  allowed: staged experimental records and cold materialization
  forbidden: claiming full LLM readiness without eval
```

The shared contract is semantic, not identical storage.

## Migration Phases

### Phase 0: Inventory and Invariants

Status: done.

Deliverables:

- this plan;
- `field_core` placeholder module with no runtime behavior change;
- invariants document in module comments;
- tests proving no command output changes yet.

Acceptance:

```bash
cargo fmt --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
scripts/test-local.sh
scripts/test-edge-cases.sh
nanda-dogfood . --refactor-plan --boundary-economics --format json
nanda-self-check
```

### Phase 1: Type Shell

Status: done in `src/field_core/`.

Create neutral types only:

- `FieldBasis`;
- `FieldAxis`;
- `FieldVectorView`;
- `FieldPeak`;
- `FieldState`;
- `FieldLensKind`;
- `AntiWaveLane`;
- `FeedbackTrace`.

No existing scoring code moves yet.

Acceptance:

- unit tests for size/serialization where applicable;
- no behavior diff in `nanda-search`, `nanda-pack6m`, `nanda-llmwave-big`.

### Phase 1a: Shared Compute Primitives

Status: done in `src/field_core/`.

The first real compute layer exists without moving domain scoring yet:

- `FieldVector1024`: deterministic 1024-dimensional sign projection;
- triad projection: `subject/relation/object` plus optional `route/group`;
- bind and bundle operations;
- cosine, energy and signature helpers;
- conservative peak detector;
- route/group/role/polarity/evidence lens operation;
- local anti-wave application with before/after alignment;
- coherence summary over field, reference peak and foreign pull.

Claim boundary:

- this is not a full LLM;
- this is not proof of nonlinear memory;
- this does not yet replace structural, packed, or LLMWave scoring loops;
- this is the shared physics substrate they can migrate toward.

Acceptance:

```bash
cargo test field_core --all-targets --all-features
```

### Phase 2: Structural Adapter

Status: done as a read-only adapter in `src/field_core/adapters.rs`.

Wrap existing `map_gate`/`search` outputs in `field_core` vocabulary:

- map existing `peak_decision` to `FieldPeak`;
- map `field_state_machine` to `FieldState`;
- map `foreign_pull` to coherence/pull metrics;
- map negative lanes to `AntiWaveLane`.

Do not rewrite scoring yet.

Acceptance:

- all existing eval/waw suites still pass;
- output can include new compatibility section, but old fields remain.

### Phase 3: Packed 6M Adapter

Status: done as a read-only adapter in `src/field_core/adapters.rs`.

Map packed hot records without changing their memory layout:

- `PackedWave1024` implements/exports `FieldVectorView`;
- `PackedSupportField` maps to support/anti-support field summary;
- `PackedAxisPeak` maps to `FieldPeak`;
- packed anti lanes map to `AntiWaveLane`.

Hard rule:

```text
no JSON, no strings, no heap/hashmap in packed inner loops
```

Acceptance:

- packed struct sizes unchanged;
- bench6m hot-cycle results remain within noise;
- no packed runtime budget regression.

### Phase 4: LLMWave Big Adapter

Status: done as a read-only adapter in `src/field_core/adapters.rs`.

Map L2/L3/schema/surface/feedback reports to unified field language:

- L2 word field -> surface/token projection;
- L3 schema field -> schema/role/route projection;
- lens scan -> `FieldLens`;
- feedback memory -> `FeedbackTrace`;
- evidence proof -> peak permission contract.

Acceptance:

- claim boundaries remain conservative;
- no new "LLM ready" or "nonlinear proof" claims;
- existing LLMWave Big tests pass.

### Phase 5: Unified Field Report

Status: done through `nanda field-report`, `scripts/nanda-field-report`, and
serve `field_report`.

The read-only command is:

```bash
nanda field-report --from search-result.json --format json
nanda field-report --from pack6m-result.json --format json
nanda field-report --from llmwave-big-result.json --format json
```

It should answer:

- what basis was used;
- what lens was applied;
- what peak won;
- what anti-wave suppressed;
- what evidence permits or blocks answer/use;
- whether this field is structural, packed, or cognitive.

Acceptance:

- report works across all three adapters;
- no source command behavior is broken.

### Phase 6: Embedded Unified Field Outputs

Status: done for the current report layer.

The first source commands now embed `unified_field` directly in their JSON
outputs:

- `nanda search` -> structural unified field;
- `nanda pack6m` -> packed unified field;
- `nanda llmwave-big * --format json` -> cognitive unified field through the
  shared LLMWave report printer.

The embedded field is report-layer only. It does not change original scoring,
packed hot loops, or LLMWave claim boundaries.

Acceptance:

```bash
scripts/test-local.sh
scripts/test-edge-cases.sh
```

### Phase 7: Shared Field Pass

Status: done in `src/field_core/pass.rs` and exposed through
`nanda field-audit`.

The unified report now projects each family into one shared pass contract:

- `FieldRecord`;
- `FieldPassInput`;
- `FieldAntiWaveLane`;
- `FieldPassReport`;
- one conservative PASS/WATCH/VETO decision shape.

This is not yet the only engine in the repository. Structural, packed, and
LLMWave modules still own their domain-specific scoring/runtime paths. The
field pass is the shared bridge and audit target.

Acceptance:

```bash
nanda field-audit --format json
```

must report:

```text
one_field_pass = true
field_core_as_sole_engine = false
llm_ready = false
```

### Phase 8: Feedback Memory Delta

Status: done locally in `src/field_core/feedback.rs`.

Feedback is now represented as a replayable memory delta before the next
field pass:

- accepted feedback emits a local reinforcement delta;
- rejected feedback emits a local suppression/anti-wave delta;
- WATCH feedback is preserved as observation only;
- deltas are local unless explicitly promoted by a higher-level memory store.

The output field is `unified_field.memory_delta`, and the same deltas are
applied before `unified_field.field_pass` is computed.

Claim boundary:

- this is feedback replay, not gradient training;
- this is local field memory, not proof of nonlinear memory;
- this does not make the system a chat LLM.

Acceptance:

```bash
cargo test field_core --all-targets --all-features
scripts/test-local.sh
nanda field-audit --format json
```

### Phase 9: Semantic Field Equivalence

Status: done through `nanda field-equivalence` and
`scripts/nanda-field-equivalence`.

The equivalence command checks that structural, packed, and cognitive reports
all project into the same shared report contract:

- `compute_probe`;
- `memory_delta`;
- `field_pass`;
- family-specific `field_pass.family`;
- conservative claim boundary.

This is semantic contract equivalence, not proof that the three engines compute
identical domain scores.

Acceptance:

```bash
nanda field-equivalence \
  --structural-from search-result.json \
  --packed-from pack6m-result.json \
  --cognitive-from llmwave-big-result.json \
  --format json
```

### Phase 10: Boundary Closure

Status: done as `KEEP`.

The route-scoped boundary check did not justify splitting the field adapter
layer further. That is an accepted outcome:

```text
NO EVIDENCE => NO CUT
```

The current closure state is:

- shared vocabulary exists;
- shared compute primitives exist;
- shared `FieldPass` exists;
- feedback emits local replayable `FieldMemoryDelta`;
- structural, packed, and cognitive reports pass the equivalence gate;
- report-module extraction is not required until boundary evidence changes.

This closes the unified-field bridge refactor. It does not claim that
`field_core` is the sole runtime engine.

## Level 2 Runtime Migration

Level 2 starts the migration from a shared bridge to a shared compute runtime.
The rule is:

```text
adapter -> dual-run -> equivalence eval -> cutover -> audit
```

### Phase 11: Field Runtime Contract

Status: active.

Every unified field report now exposes `runtime_contract`:

- input: `FieldPassInput`;
- output: `FieldPassReport`;
- role: shared semantic pass;
- `field_core_as_sole_engine = false`;
- domain engine preserved until cutover evidence exists.

This is the runtime contract, not yet the runtime cutover.

### Phase 12: Structural Dual-Run

Status: active for `nanda search`.

`nanda search --format json` now emits `field_runtime`:

- old structural peak/verdict/state/safe flag;
- field-core peak/verdict/state/safe flag;
- peak match;
- state-family match;
- not-more-permissive check;
- `cutover_ready`.

Current expected behavior:

- focused route-trap can be `cutover_ready=true`;
- noisy/contested/reversed/thin cases can be `cutover_ready=true` only when
  `state_hint` preserves the unsafe state family and the field pass is not more
  permissive than the domain engine.

Claim boundary:

- field runtime dual-run must never make the field more permissive than the
  existing domain engine;
- `field_safe_to_answer` remains blocked by claim boundary.

### Phase 13: Packed Dual-Run

Status: active for `nanda pack6m`.

`nanda pack6m --format json` now emits `field_runtime`:

- old packed route peak/state/verdict/safe flag;
- field-core peak/state/verdict/safe flag;
- not-more-permissive check;
- `cutover_ready`.

Packed remains a protected hot-core exception:

- no JSON/string/heap is introduced into the packed inner loop;
- dual-run is report-layer evidence;
- `packed_hot_core_exception = true` remains in audit until a zero-cost
  `FieldRecordView` and benchmark guard exist.

### Phase 14: Cognitive Dual-Run

Status: active for `nanda llmwave-big * --format json`.

The shared LLMWave report printer now emits `field_runtime` for cognitive
reports:

- cognitive verdict/state/safe flag;
- shared field pass peak/state/safe flag;
- not-more-permissive check;
- `cutover_ready`.

The cognitive path remains not-LLM:

- `field_safe_to_answer` stays false unless claim boundaries change through
  future eval evidence;
- `*_NOT_ANSWER`, `*_NOT_CHAT`, `*_NOT_LLM`, and `*_NOT_FIELD_MATURE` states
  are mapped to review-only field state;
- local answer candidates may be focused but still not broad chat readiness.

### Phase 15: Runtime Audit Closure

Status: done as semantic runtime, not sole engine.

The current audit target is:

```text
field_core_as_semantic_engine = true
field_core_as_sole_engine = false
packed_hot_core_exception = true
```

This is the honest Level 2 closure:

- structural outputs run shared dual-run;
- packed outputs run shared dual-run without changing the packed hot loop;
- cognitive outputs run shared dual-run without claiming LLM/chat readiness;
- lens, anti-wave, and memory delta contracts are unified at `field_core`;
- full sole-engine cutover is blocked until packed zero-cost views and
  benchmark guards exist.

### Phase 16: Packed FieldRecordView

Status: done.

`nanda_6m::PackedFieldRecordView<'a>` is a borrowed typed view over
`PackedTriad32`:

- no JSON;
- no string;
- no heap/hashmap;
- no copied record payload;
- exposes numeric subject/relation/object/route/group/confidence/polarity axes.

`nanda pack6m --format json` reports the view contract under
`field_record_view`. This is a hot-core storage/readiness contract, not a hot
cutover by itself.

### Phase 17: Packed Cutover Bench Guard

Status: done as a guard, not permission.

`nanda bench6m --format json` now reports `field_runtime_cutover_guard`:

- required packed view version;
- whether this bench run contains hot-cycle or active-core evidence;
- explicit blockers for sole-engine cutover;
- `field_core_as_sole_engine_allowed = false`.

This prevents the project from silently turning report-layer dual-run into a
hot packed cutover without benchmark evidence and an explicit follow-up change.

### Phase 18: Structural Cutover Suite

Status: done as an explicit structural proof gate, not global cutover.

`nanda field-cutover --suite structural-standard --format json` now runs the
standard structural fixtures directly. It also accepts structural
`nanda search` outputs through repeated `--structural-case` arguments for
custom suites. Both paths evaluate the shared runtime contract across a suite:

- peak match;
- state-family match;
- field pass is not more permissive than the structural domain engine;
- every case is `cutover_ready`.

The suite is intended to include at least:

- focused route-trap;
- contested/noisy field;
- reversed polarity stop;
- thin/negative-lane field.

Passing the suite means:

```text
field_core_as_structural_engine_candidate = true
field_core_as_sole_engine_allowed = false
```

This is deliberately narrower than full sole-engine cutover. Packed remains a
protected hot-core exception, and cognitive remains not-LLM/not-chat until
separate eval evidence changes those claim boundaries.

### Phase 19: Live Cutover Audit Snapshot

Status: done.

`nanda field-audit --format json` now embeds a live
`structural_cutover_suite` snapshot built from the same `structural-standard`
cases as `nanda field-cutover`. The audit no longer only says that the suite is
available; it reports the current suite state and mirrors
`structural_cutover_suite_pass` into top-level acceptance.

This keeps the audit honest:

```text
structural_cutover_suite_pass = true
field_core_as_sole_engine = false
```

The audit can therefore be used as a quick health report for the unified field,
but still does not grant global cutover permission.

### Phase 20: Structural Field Engine Candidate Mode

Status: done as participant mode, not behavior cutover.

`nanda search` now accepts:

```bash
nanda search packet.json --input-format json --field-engine legacy
nanda search packet.json --input-format json --field-engine shadow
nanda search packet.json --input-format json --field-engine candidate
```

The output includes `field_engine`:

- `legacy`: structural-domain engine remains selected, field is reported by
  dual-run only;
- `shadow`: field explicitly participates as a shadow candidate, but selected
  output remains structural-domain;
- `candidate`: if the dual-run case is `cutover_ready` and the field is not
  more permissive, `selected_engine=field-core-candidate`.

Important boundary:

```text
top_level_behavior_changed = false
field_core_as_sole_engine = false
```

This is the first step where the field becomes an explicit structural compute
participant. It still does not replace the structural search top-level verdict
or peak until a separate structural cutover phase changes that contract.

### Phase 21: Structural-Only Field Cutover Mode

Status: done as explicit opt-in cutover.

`nanda search` now also accepts:

```bash
nanda search packet.json --input-format json --field-engine cutover
```

If `field_runtime.cutover_ready=true` and
`field_runtime.field_not_more_permissive=true`, the top-level structural search
contract is rewritten from the field-core candidate:

- `top_peak`;
- `verdict`;
- `field_state`;
- `safe_to_answer`.

The output includes `field_cutover` with the old and new values:

```text
field_cutover.applied = true
field_core_as_structural_sole_engine = true
field_core_as_sole_engine = false
```

This means field-core is the opt-in sole structural engine for that search
result. It is not global sole-engine permission: packed remains a protected
hot-core exception and cognitive remains not-LLM/not-chat.

### Phase 22: Packed Field Engine Guard

Status: superseded by the packed-only field-core cutover step below.

`nanda pack6m` now accepts the same field-engine vocabulary:

```bash
nanda pack6m packet.json --input-format json --field-engine legacy
nanda pack6m packet.json --input-format json --field-engine candidate
nanda pack6m packet.json --input-format json --field-engine cutover
```

The output includes `packed_field_engine`:

- `legacy`: packed hot-core remains selected;
- `candidate`: field-core can be reported as a packed engine candidate when
  `field_runtime.cutover_ready=true`, but top-level packed behavior does not
  change;
- `cutover`: records the cutover request and blocks it through the hot-core
  guard.

Historical invariant before the packed-only cutover superseded this phase:

```text
packed_field_engine.selected_engine = packed-hot-core
packed_field_engine.cutover_applied = false
packed_field_engine.field_core_as_sole_engine = false
packed_field_engine.field_core_as_packed_hot_engine = false
packed_field_engine.hot_core_guard.packed_hot_core_exception = true
```

That older invariant kept packed as the zero-cost/hot-loop exception while the
unified field learned to reason about packed readiness in the same engine
vocabulary as structural search.

### Latest Implemented Step: Packed-Only Field-Core Cutover

Packed runtime now has an explicit field-core cutover path. The cutover is
still scoped to the packed family only; it does not claim global
`field_core_as_sole_engine`, LLM readiness, or nonlinear memory proof.

`nanda pack6m packet.json --input-format json --field-engine cutover` now
reports:

```text
packed_field_engine.selected_engine = field-core-packed-cutover
packed_field_engine.cutover_applied = true
packed_field_engine.field_core_as_packed_hot_engine = true
packed_field_engine.field_core_as_packed_sole_engine = true
packed_field_engine.hot_core_guard.satisfied_by_typed_packed_decision = true
packed_field_engine.claim_boundary.global_sole_engine = false
packed_field_engine.claim_boundary.llm_ready = false
packed_field_engine.claim_boundary.nonlinear_memory_proven = false
```

The top-level packed peak decision is updated only through
`field_core::apply_packed_field_cutover`, using the typed hot-safe
`nanda_6m::evaluate_packed_peak_decision` contract. A thin packed peak remains
`WATCH` and `safe_to_answer=false`.

### Phase 23: Cognitive Field Engine Guard

Status: superseded by the cognitive-only field-core cutover step below.

LLMWave-Big JSON reports emitted through the shared report layer now include
`cognitive_field_engine`.

The guard says:

```text
cognitive_field_engine.field_core_as_semantic_engine = true
cognitive_field_engine.field_core_as_sole_engine = false
cognitive_field_engine.field_core_as_chat_engine = false
cognitive_field_engine.field_core_as_llm = false
cognitive_field_engine.cutover_applied = false
```

This older guard allowed the unified field to participate as a
semantic/cognitive projection, but it did not yet make cognitive a family-scoped
field-core engine.

### Latest Implemented Step: Cognitive-Only Field-Core Cutover

LLMWave-Big reports now expose a cognitive-only field-core cutover when the
cognitive dual-run is cutover-ready and not more permissive than the domain
report. Domain-specific top-level verdicts are preserved for human/API
compatibility; the selected field-core state is exposed in
`cognitive_field_engine` and `cognitive_field_cutover`.

```text
cognitive_field_engine.selected_engine = field-core-cognitive-cutover
cognitive_field_engine.cutover_applied = true
cognitive_field_engine.field_core_as_cognitive_sole_engine = true
cognitive_field_engine.field_core_as_chat_engine = false
cognitive_field_engine.field_core_as_llm = false
cognitive_field_cutover.top_level_domain_contract_preserved = true
```

This completes the unified-field engine cutover across structural, packed, and
cognitive families without claiming broad chat, LLM readiness, or nonlinear
memory proof.

### Phase 24: Three-Family Field Engine Audit

Status: done as the current unified field closure.

`nanda field-audit --format json` now reports `field_engine_contract` across
all three families:

- structural: `structural-field-engine-v1`, opt-in cutover may be allowed when
  the structural standard suite passes;
- packed: `packed-field-engine-guard-v1`, explicit packed-only cutover is
  allowed through the typed packed decision core;
- cognitive: `cognitive-field-engine-guard-v1`, cognitive-only cutover is
  allowed while the LLM/chat claim remains blocked.

Acceptance fields:

```text
three_family_engine_contract = true
structural_cutover_mode_available = true
packed_field_engine_guard = true
packed_cutover_blocked_by_hot_guard = false
packed_field_core_as_sole_engine = true
cognitive_field_engine_guard = true
cognitive_cutover_blocked_by_claim_guard = false
cognitive_field_core_as_sole_engine = true
field_core_as_sole_engine = true
llm_ready = false
nonlinear_memory_proven = false
```

This is the present answer to "do we have one field?": yes, one field
vocabulary/pass/engine contract now covers structural, packed, and cognitive
reports. Structural and packed now have family-scoped field-core sole-engine
paths, and cognitive has a cognitive-only field-core sole-engine path. Global
field-core sole-engine is now true, while LLM/chat readiness and nonlinear
memory proof remain false behind claim gates.

### Phase 25: Shared Field Engine Policy Owner

Status: done.

The engine-policy layer moved into `src/field_core/engine.rs`.

The shared owner is:

```text
field_core::engine::FieldEngineDecision
```

It now owns the common decision shape for:

- structural `field_engine`;
- packed `packed_field_engine`;
- cognitive `cognitive_field_engine`.

Domain modules now call the shared policy instead of hand-building separate
engine decisions:

- `search.rs` calls `field_core::structural_field_engine_decision`;
- `pack6m.rs` calls `field_core::packed_field_engine_decision`;
- `llmwave_big/report.rs` calls `field_core::cognitive_field_engine_decision`.

`nanda field-audit --format json` reports:

```text
field_engine_contract.policy_owner = field_core::engine::FieldEngineDecision
acceptance.field_engine_policy_in_field_core = true
```

This closes the first "real shared physics" step: the engine decision contract
is no longer copied across the three field families.

## Refactor Gates

Before each phase:

```bash
nanda-dogfood . --refactor-plan --boundary-economics --format json
nanda-map-code <target-file-or-dir> --format json
```

If target file is HIGH risk:

- introduce a module boundary first;
- avoid semantic changes in the same commit.

After each phase:

```bash
cargo fmt
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
scripts/test-local.sh
scripts/test-edge-cases.sh
scripts/install-local.sh
nanda-self-check
```

Guard-diff rules:

- use route-specific action id for code changes;
- use `shared.version_bump_contract` only for pure version metadata;
- no PASS if diff crosses routes without explicit shared contract.

## Implemented Code Step

The first code step was intentionally small:

```text
added src/field_core/*
added src/field_adapters/mod.rs
added nanda field-report
added scripts/nanda-field-report
kept existing runtime behavior unchanged
tests: compile-only plus tiny type tests
```

This creates the target seam without moving scoring logic.

## Latest Implemented Step: Shared Field Operation Contracts

The unified field now owns the first shared operation contracts, not only the
report vocabulary:

```text
field_core::peak::FieldPeakInput / FieldPeakResult
field_core::coherence::FieldCoherenceInput / FieldCoherenceResult
field_core::coherence::field_verdict_for_state
field_core::anti_wave::FieldAntiWaveEffect
field_core::readout::FieldReadoutInput / FieldReadoutResult
field_core::readout::FieldLocalPathInput / FieldLocalPathResult
```

`search.rs` still emits the legacy `peak_decision` and
`field_state_machine` JSON shapes, but the actual peak/coherence/verdict
decisions are delegated to `field_core`. `model.rs::field_interpretation` and
`search.rs::coarse_to_fine_trace` are now compatibility wrappers around
`field_core::readout`. This keeps downstream compatibility while moving
ownership of the field physics into one module family.

`nanda field-audit --format json` reports:

```text
field_operation_contract.version = unified-field-operation-contract-v1
field_operation_contract.peak_owner = field_core::peak::FieldPeakResult
field_operation_contract.coherence_owner = field_core::coherence::FieldCoherenceResult
field_operation_contract.anti_wave_owner = field_core::anti_wave::FieldAntiWaveEffect
field_operation_contract.readout_owner = field_core::readout::FieldReadoutResult
field_operation_contract.local_path_owner = field_core::readout::FieldLocalPathResult
acceptance.structural_decision_uses_field_core = true
```

This is still not a global sole-engine claim. Packed and cognitive paths remain
guarded by their hot-core and LLM/chat claim boundaries.

## Latest Implemented Step: Structural Sole-Engine Cutover

Structural search now defaults to field-core cutover. Legacy structural output is
still available with `--field-engine legacy`, but the default structural CLI path
uses the field-core candidate when the structural cutover suite proves it is not
more permissive than the legacy route.

`nanda field-audit --format json` now reports:

```text
overall_state = FIELD_CORE_SOLE_ENGINE_ACTIVE_LLM_NOT_READY
acceptance.structural_field_core_as_sole_engine = true
acceptance.packed_field_core_as_sole_engine = true
acceptance.cognitive_field_core_as_sole_engine = true
field_engine_contract.structural.cutover_mode = default
field_engine_contract.structural.structural_sole_engine = true
acceptance.field_core_as_sole_engine = true
acceptance.llm_ready = false
acceptance.nonlinear_memory_proven = false
```

The last two remain false intentionally: field-core sole-engine is an engine
ownership claim, not a broad chat/LLM or nonlinear-memory proof.

## Latest Implemented Step: Nonlinear Memory Eval Harness

`nanda llmwave-big nonlinear-memory-eval --format json` now compares a
fixed-basis residual wave memory against a linear full-record baseline over the
same synthetic capacity sweep.

Current status:

```text
verdict = NONLINEAR_MEMORY_SCALE_CANDIDATE_NOT_PROVEN
aggregate.state = USEFUL_DENSITY_SCALE_CANDIDATE
aggregate.large_scale_win_rate = 1.0
claim_boundary.useful_density_candidate = true
claim_boundary.nonlinear_memory_proven = false
```

With the external fixture:

```bash
nanda llmwave-big nonlinear-memory-eval \
  --corpus examples/llmwave-big-nonlinear-memory-corpus.json \
  --format json
```

the external/noise gates report:

```text
external_corpus.state = EXTERNAL_FIXTURE_AND_NOISE_PASS
external_corpus.heldout_pass_rate = 1.0
external_corpus.negative_reject_rate = 1.0
external_corpus.noise_reject_rate = 1.0
claim_boundary.scale_amortized_nonlinear_memory_proven = true
claim_boundary.nonlinear_memory_proven = false
```

This is a deliberately narrow result. It says the fixed basis starts to win at
larger scale after basis overhead is amortized. It does not prove nonlinear
memory yet, because the full-sweep baseline gates are still stricter than the
current scale-candidate evidence.

The nonlinear-memory claim stays blocked until:

- fixed-basis memory beats the linear baseline under the configured capacity
  gates;
- bytes per useful fact improves under held-out evaluation;
- schema reuse and residual saving survive scale;
- role error rate and false positive rate stay bounded;
- external corpus and broad noise checks pass.

## Refactor Queue

Priority order:

1. Move one bounded scoring explanation from structural report text into
   `field_core` vocabulary.
2. Add a route-scoped refactor plan for any large reporting file before
   splitting it.
3. Only then consider extracting reporting submodules.

Files not to refactor first:

- `src/llmwave_big/report.rs`: large but route-scoped boundary currently says
  KEEP; split only after unified field report exists.
- `src/model.rs`: large but low current risk; do not move common triad model
  code until adapter boundaries are stable.
- `src/map_gate.rs`: central structural map; wrap first, move later.

## Success Criteria

The refactor is successful only if all are true:

- one shared field vocabulary exists in code;
- three adapters can project into it;
- one shared field pass exists and is visible in report outputs;
- feedback can emit a local memory delta that changes the next field pass;
- structural, packed, and cognitive reports pass the shared equivalence gate;
- packed hot-loop constraints are preserved;
- old outputs and tests remain compatible;
- agent-facing reports become clearer;
- no project-specific hardcode is introduced;
- no LLM/nonlinear/cache-only claim boundary is weakened.
