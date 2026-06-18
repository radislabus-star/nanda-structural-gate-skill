# NANDA-6M Cache-Resident Packed Core

Status: packed replay contract with diagnostic compute sweep.

NANDA-6M is the planned cache-resident core for NANDA. It is not another
command added to the current reference engine. The current v3.0 Rust CLI is a
correctness and UX engine: it parses JSON/Markdown, owns strings, builds maps,
explains decisions, and is allowed to be convenient. NANDA-6M is the opposite
layer: a packed hot path with a hard memory budget.

## What "Full 6M Model" Means

In this project, a "full NANDA-6M model" does not mean a general-purpose
standalone LLM compressed into 6 MiB. That would require token embeddings,
many learned transformer/SSM/MLP layers, generation logits, decoding, and a
training pipeline. NANDA-6M is a different object: a cache-resident structural
field that can sit next to an LLM and decide whether a set of relations forms a
stable route.

A full NANDA-6M structural model must include all of these inside the 6 MiB hot
contract:

1. Packed fact memory: fixed `PackedTriad32` relation records.
2. Query projection: convert the active candidate/query into a compact wave.
3. Route/group centroids: keep focused structural centers resident.
4. Constructive lanes: reinforce accepted route/group/support shapes.
5. Destructive lanes: suppress known false shortcuts without killing a topic.
6. Replay firewall: distinguish stable peaks from intervention-dependent peaks.
7. Hierarchical/local gates: split oversized maps and accept only local PASS.
8. Top-k decision packet: return peak, margin, support, anti-support, and state.
9. Budget refusal: return `FOCUS_REQUIRED`, `SPLIT_REQUIRED`, or
   `SPILL_REQUIRED` instead of silently leaving cache.
10. Cold bridge contract: map text/evidence to IDs before hot execution and map
    IDs back to explanation after hot execution.

Current v3.0 already has the cold bridge, budget planner, packed records,
packed projection, centroids, lanes, replay, and firewall diagnostics. It is
not full yet because the complete search/hierarchy loop is still partly
assembled by the dynamic CLI layer, and the hot core does not yet own the whole
fixed-array top-k route decision.

The short definition:

```text
full NANDA-6M = relation memory + wave projection + centroids + lanes +
replay firewall + hierarchical route decision, all inside fixed 6 MiB arrays.
```

Anything involving natural-language generation remains outside the 6 MiB hot
core. The LLM writes and reads language; NANDA-6M checks the structural field.

The core rule is simple:

```text
hot structural reasoning must fit inside 6 MiB, or the core must refuse the
packet and ask for focus/split.
```

## Goal

NANDA-6M should make the "interference peak" a real local computation:

1. Pack relation facts into compact IDs and fixed records.
2. Keep route/group centroids resident in cache.
3. Apply constructive and destructive lanes without heap allocation.
4. Return a small decision packet: peak, margin, suppression, branch verdict.
5. Let the cold layer map IDs back to text, evidence, JSON, and explanations.

This makes NANDA usable as a hard local gate for LLM work: the LLM may carry a
large, noisy superposition of possible meanings, while NANDA-6M checks whether
one structural route forms a stable peak inside a small processor-local field.

## Non-Goals

NANDA-6M is not:

- a standalone LLM;
- a vector database;
- a text store;
- a source/evidence archive;
- a JSON parser;
- a replacement for the v3.0 reference engine;
- a place for `String`, `Vec` per record, `HashMap`, `BTreeMap`, or serde in
  the query loop.

Those belong to the cold layer.

## Hot And Cold Boundary

Cold layer responsibilities:

- parse JSON/Markdown;
- normalize paths, routes, groups, roles, and evidence;
- build dictionaries from text labels to integer IDs;
- choose a focused packet when the source corpus is too large;
- map output IDs back to human-readable explanations;
- keep source snippets and audit trails.

Hot layer responsibilities:

- accept already-packed arrays;
- run fixed-budget interference, centroid, lane, and hierarchy checks;
- allocate nothing during query;
- return only packed IDs, scores, states, and small top-k traces.

Hard hot-path bans:

```text
String
serde_json
HashMap / BTreeMap
Vec allocation during query
per-triad heap objects
text evidence
unbounded route/group growth
silent spill into RAM
```

If the packet cannot fit, the hot core returns `FOCUS_REQUIRED`,
`SPLIT_REQUIRED`, or `SPILL_REQUIRED`. It must not silently become a large RAM
engine.

## Memory Budget

Hard budget:

```text
6 MiB = 6,291,456 bytes
```

Initial packed layout:

```text
Header/control             16,384 B
Triad arena             2,097,152 B
Centroid arena          2,097,152 B
Lane arena              1,048,576 B
Query/workspace/top-k     786,432 B
Index/calibration/stats   245,760 B
-----------------------------------
Total                   6,291,456 B
```

This is the NANDA-6M law. Any implementation must be able to print this budget
and prove `used_bytes <= 6,291,456` for the hot core.

v20 adds a stricter runtime attach contract on top of the broad arena budget.
The full arena can hold 65,536 packed triads, but the current hot-cycle
algorithm also needs temporary score and bucket arrays inside the 786,432 byte
workspace. Therefore a packet may fit the broad 6 MiB arena while still
returning `FOCUS_REQUIRED` for one hot-cycle attach. This is intentional: the
runtime must refuse unfocused work instead of silently spilling score buffers
into normal RAM.

The v20 runtime states are:

```text
PACKED_RUNTIME_READY
FOCUS_REQUIRED
SPLIT_REQUIRED
SPILL_REQUIRED
EMPTY_MEMORY
EMPTY_QUERY
WORKSPACE_TOO_SMALL
```

`PackedHotCore::attach` is the hot boundary. It accepts already packed memory
and preallocated workspace slices, validates the shape, and only then allows
`run_query`.

## Capacity Target

The first NANDA-6M target is:

```text
triads:     65,536 packed triads
centroids:   2,048 route/group centroids
lanes:      16,384 constructive/destructive lanes
wave_dim:    1,024 i8 dimensions per centroid
```

This is not "all memory of the world." It is the focused active field. Large
corpora must be reduced by dataset doctor, route balancing, query triads, and
coarse-to-fine focus before they enter NANDA-6M.

## Packed Records

`PackedTriad32` is the core fact record. It is exactly 32 bytes:

```text
subject_id:    u32   4 B
object_id:     u32   4 B
evidence_ref:  u32   4 B
wave_seed:     u32   4 B
relation_id:   u16   2 B
route_id:      u16   2 B
group_id:      u16   2 B
role_pack:     u16   2 B
flags:         u16   2 B
lane_hint:     u16   2 B
check:         u16   2 B
confidence:    u8    1 B
polarity:      u8    1 B
-----------------------
total                32 B
```

The hot core does not store the subject string, relation string, object string,
or evidence text. It stores IDs only. The cold layer owns dictionaries and
evidence.

`PackedLane64` is the first lane target. It is exactly 64 bytes:

```text
support_mask_a:   u64
support_mask_b:   u64
anti_mask_a:      u64
anti_mask_b:      u64
lane_id:          u32
target_route:     u16
target_group:     u16
target_relation:  u16
accepted_count:   u16
rejected_count:   u16
margin_hint:      i16
action:           u8     # suppress, boost, veto, watch
strength:         u8
reserved:         [u8; 14]
```

The field order is part of the contract. `src/nanda_6m.rs` uses `repr(C)` and
unit tests to prove that `PackedTriad32`, `PackedCentroid1024`, and
`PackedLane64` have the expected byte sizes.

Lanes should be local to a reading shape, not global topic killers. A negative
lane should suppress "this support shape means the wrong route," not "never
retrieve this route."

## Wave Representation

Initial core wave dimension:

```text
WAVE_DIM = 1024
centroid = [i8; 1024] = 1,024 B
```

The centroid arena therefore holds 2,048 centroids in 2 MiB. A later 2,048-dim
core is possible, but it would cut centroid capacity in half or expand the
budget. NANDA-6M starts with 1,024 dims because cache residence is more
important than a prettier dimension count.

Triads do not store full waves. They store `wave_seed`, relation/route/group
IDs, polarity, role flags, and confidence. The hot core projects a triad into a
working wave on demand by deterministic signed hashing and role binding.

The first packed diagnostic path keeps memory/source records separate from the
candidate/query wave. It scores the query wave against memory route/group
centroids, then reports per-triad support and anti-support for the top packed
axis. This lets the core distinguish "there is no peak" from "there is a thin
peak because constructive and destructive contributions nearly cancel."

`packed_lanes` is the first `PackedLane64` bridge. It is preview-only: it
builds a lane-shaped mask over the current anti-support records and reports the
possible `net_dot` change if that destructive contribution were suppressed.
This proves the byte-level lane can target a reading shape before the hot loop
learns and applies persistent lanes.

The persistent lane identity is not the record mask. `packed_lane_keys` stores
a cold stable signature over the support/anti-support shape. The current focus
packet compiles that key into a `PackedLane64` mask for the local record window.
This keeps the 6 MiB arena cache-resident and avoids treating transient record
indexes as durable memory.
The key signature is based on stable projected shape fields such as
`wave_seed`, polarity, and confidence, not on current dictionary IDs or record
indexes.
`packed_lane_store` reports the compiled hot arena cost: each runtime lane is
64 bytes, so the 1 MiB lane arena holds 16,384 compiled lanes. This is the
rational 6M representation; cold signatures can live outside the hot arena and
compile into masks for the current focused packet.

`packed_lane_replay` connects feedback memory to the packed core. It matches
negative/positive shortcut memory against current stable lane keys, compiles
matched keys into current-window `PackedLane64` masks, and reports replayed
`before_net_dot -> after_net_dot` without treating the result as final answer
permission.

v3.0 keeps the observer-to-compute sweep. Replay is evaluated at observer,
soft-touch, medium-touch, and full-touch strengths. A stable peak should improve
under soft touch; a peak that only appears under full touch is marked as
strongly intervention-dependent. `computational_effect` may say that replay is
ready to shape the packed field, but `safe_to_answer` remains false until the
normal structural gate accepts the result.

`packed_replay_decision` is the replay firewall. It compares the raw
`peak_decision` with the replay-adjusted field and classifies the result as
`STABLE_WITH_REPLAY`, `REPLAY_RESCUED_THIN_FIELD`,
`REPLAY_DESTABILIZED_FIELD`, `REPLAY_TOO_STRONG_REQUIRED`, or
`NO_REPLAY_EVIDENCE`. In v3.0 this decision is produced by
`nanda_6m::evaluate_replay`, a typed hot-compatible function. The CLI converts
JSON diagnostics into packed enum inputs, calls the typed core, and converts the
result back to JSON for agents. Replay now influences the decision surface
without putting JSON, strings, maps, or serde in the hot decision path.

`packed_lane_application` is the first single-pass applied-lane diagnostic. It
applies the preview mask to the support-map and reports whether the adjusted
field becomes a focused candidate. This is still not an answer gate:
`safe_to_answer` remains false until persistent lanes are learned and replayed
inside the cache-resident hot loop.

Workspace may use wider accumulators:

```text
query_wave:       [i16; 1024]
scratch_wave:     [i16; 1024]
score buffers:    fixed arrays
top-k buffers:    fixed arrays
```

Centroids stay compact as `i8`; accumulators may use `i16` or `i32` to avoid
saturation during local scoring.

## Hot Query Pipeline

The hot path should be deterministic and allocation-free:

1. Reset fixed workspace.
2. Project query triads into `query_wave`.
3. Score route centroids.
4. Score group centroids inside the best routes.
5. Apply negative lanes to local false shortcuts.
6. Apply positive lanes to accepted route/group/support shapes.
7. Select top-k peaks using fixed buffers.
8. Run hierarchical gate over linked route/group branches.
9. Return compact output IDs, scores, state, and safety flags.

The cold explain layer then turns IDs into human-readable output.

## States

NANDA-6M must distinguish computation success from answer safety:

```text
FITS_L3
FOCUS_REQUIRED
SPLIT_REQUIRED
SPILL_REQUIRED
NOISY_FIELD
LOW_MARGIN
PEAK_FOUND
STRUCTURALLY_ACCEPTED
REPAIR_REQUIRED
VETO
WATCH
```

`PEAK_FOUND` is not enough. A peak can be a useful retrieval hint and still be
unsafe to answer from. `STRUCTURALLY_ACCEPTED` requires a stable peak, no
foreign pull, no active veto lane, and passing linked branches.

## Why This Is Different From v3.0

Current v3.0 reference behavior is intentionally expressive:

```text
JSON packet -> Rust structs with String fields -> maps/vectors -> JSON report
```

NANDA-6M behavior must be:

```text
packed packet -> fixed arrays -> interference field -> compact decision
```

v3.0 proves what to compute. NANDA-6M proves that the computation can be kept
small, local, predictable, and fast enough to act as an always-on agent gate.

## Acceptance Tests

The first implementation must pass these before it is trusted:

1. Budget proof: `used_bytes <= 6,291,456`.
2. No heap allocation inside hot query.
3. Route-splice fixture returns `REPAIR_REQUIRED` or `VETO`.
4. HGate size-only fixture returns `STRUCTURALLY_ACCEPTED` only when local
   branches pass.
5. Negative shortcut fixture suppresses the rejected shortcut, not the whole
   topic.
6. Positive shortcut fixture boosts the accepted route/group/support shape.
7. Large unfocused corpus returns `FOCUS_REQUIRED`, not a fake PASS.
8. Focused corpus returns stable peaks with better margin than lexical
   baseline.
9. Linux CI must cover deterministic parity with v3.0 fixtures.
10. Microbench must report bytes used, peak count, query time, and whether the
    packet stayed in the 6 MiB budget.

## Migration Plan

Phase 0: this design contract.

Phase 1: add a budget planner only. It packs nothing yet; it estimates whether
an existing packet would fit the NANDA-6M limits and explains what must be
focused or split.

Phase 2: introduce a separate Rust module/crate for packed types. It must not
reuse the v3.0 dynamic structs in the hot path.

Phase 3: implement packed projection and centroid scoring with parity fixtures.

Phase 4: implement local positive/negative lanes and hierarchical branch
decisions.

Phase 5: benchmark and profile. SIMD can be added only after the packed
contract is stable.

## Open Design Questions

These must be resolved before implementation:

- fixed centroid split: route vs group vs role, or one shared centroid arena;
- exact i8 saturation policy for centroid updates;
- how many top-k peaks are useful inside the hot return packet;
- whether `evidence_ref` points to a cold table row or a compact source rank;
- whether branch hierarchy runs fully hot or partly in the cold orchestrator;
- how strict the no-allocation proof must be in the first release;
- how to expose `FOCUS_REQUIRED` so agents do not treat it as failure.

## Design Decision

The next engineering step is not another search flag. The next step is a
NANDA-6M budget planner and packed data model. If a feature cannot state its
byte cost, it does not enter the hot core.
