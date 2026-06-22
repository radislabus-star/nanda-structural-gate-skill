# Unified Field Refactor Plan

Status: implemented as read-only unified field projection plus first shared
compute primitives.

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

Status: done through `nanda field-report` and `scripts/nanda-field-report`.

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

## Refactor Queue

Priority order:

1. Move one small read-only adapter calculation onto `FieldVector1024` and prove
   output compatibility.
2. Add real field-report fixtures from live command output, not only synthetic
   shapes.
3. Add `nanda-serve` support for `field_report`.
4. Add optional `unified_field` sections to search/pack6m/llmwave-big outputs
   after compatibility review.
5. Only then consider splitting large reporting files.

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
- packed hot-loop constraints are preserved;
- old outputs and tests remain compatible;
- agent-facing reports become clearer;
- no project-specific hardcode is introduced;
- no LLM/nonlinear/cache-only claim boundary is weakened.
