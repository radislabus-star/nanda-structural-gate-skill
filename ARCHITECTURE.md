# Architecture

This document fixes the first checker architecture before implementation.

The purpose is not to build an open-ended "thinking cell". The purpose is to
build a deterministic structural gate that can later swap its scoring core from
stub to wave/VSA sparse-triad inference.

## Layer Contract

```text
raw task
  -> agent/extractor
  -> triad packet
  -> encoder
  -> sparse-triad wave core
  -> baseline comparators
  -> verdict report
```

NANDA starts after extraction. It should not parse raw language in the first
version.

## Inputs

The checker accepts one task packet.

The packet has two triad groups:

```text
triads           - source/evidence structure
candidate_triads - structure extracted from the answer being checked
```

Hard first limits:

```text
entities:          32 max
roles:             16 max
relations:         32 max
triads:            64 max
routes:             8 max
evidence_refs:     64 max
candidate_answers:  4 max
```

The first useful target is smaller:

```text
entities:          16
roles:              8
relations:         16
triads:            32
routes:             4
evidence_refs:     32
candidate_answers:  2
```

If a task exceeds the hard limits, the skill must split the task into local
subgraphs and return `WATCH` until each subgraph is checked.

## Output

The checker returns one verdict:

```text
PASS  - bindings are stable and baselines do not contradict them
WATCH - incomplete, weak, too large, or not yet verified
VETO  - structural conflict or role/route contradiction detected
```

Minimum report fields:

```text
verdict
complexity_score
stable_triads
weak_triads
conflicts
evidence_gaps
baseline_summary
trace_path
```

The default CLI output should be short. Detailed traces stay in local files.

## Implementation Surface

The shipped runtime core is a Rust binary:

```text
src/main.rs -> target/release/nanda -> skill/bin/nanda
```

The files in `nanda-structural-gate/scripts/` are compatibility wrappers for
stable command names such as `nanda-check`, `nanda-gate-md`, `nanda-split`,
and `nanda-split-md`.

Machine workflows should prefer JSON packets:

```text
nanda map packet.json --input-format json
nanda comb packet.json --input-format json --depth 2 --out-dir comb/
nanda split packet.json --input-format json --by linked-group --out-dir split/
nanda check --triads split/route.json
nanda dataset-doctor index.json --input-format json
nanda feedback search.json --decision reject --out .nanda/reject.json
nanda index memory.json .nanda/reject.json --out .nanda/index.json
nanda eval --suite examples/eval-corpus.json
nanda waw --suite examples/waw-corpus.json
printf '{"command":"doctor"}\n' | nanda serve
```

Markdown worksheets remain useful for human-readable extraction and review.

No secondary scripting runtime is part of the shipped checker surface.

Current core:

```text
core_version: sparse-triad-v2.2-polarity-gate
wave_dim:     1024
```

The `v2.2-polarity-gate` core keeps recursive combing, structural peak search,
reusable memory indexes, arrow-text extraction, feedback packets, regression
evaluation, and release doctor checks, then adds file-backed eval suites, a
newline-delimited JSON agent API, and field interpretation for interference
peaks. It also adds a WAW benchmark surface for cases where structural
interference must beat a lexical trap, plus a dataset-quality gate for noisy
large corpora, source/confidence weighting, auto query triads, and learning
negative lanes for rejected shortcut peaks. The current field also adds
route-balanced focus, coarse-to-fine localization, polarization lanes, and a
polarity gate:

```text
topology graph
comb tree
interference search peaks
memory index packet
separate query packet
accepted/rejected/WATCH peak feedback
peak/state eval suite
file-backed eval suite
WAW lexical-trap benchmark suite
dataset-doctor corpus immunity
negative_shortcuts
destructive_interference
source_weighting
auto_query_triads
learned negative lane penalty
route_balanced_focus
coarse_to_fine local path
polarization lane
polarization_penalty
POLARITY_REVERSED peak decision
JSONL agent serve loop
field_interpretation for search peaks
field_interpretation.corpus
self-contained doctor smoke
arrow-text extraction
source group memory
candidate group memory
route memory
group centroid summary
candidate superposition summary
interference_matrix[source_group][candidate_group]
dominant_source_group
mixed_candidate_groups
foreign_pull per candidate triad
invariant drift at deeper depths
supporting_triads / anti_triads / missing_edges
repair_tasks
```

This map is the base layer for agent API, GitHub examples, and benchmark work.
Higher-level reporting should consume the map and field interpretation instead
of re-inventing route logic.

CLI exit codes:

```text
0 - PASS
1 - VETO
2 - ERROR
3 - WATCH
```

## Trigger Threshold

The gate is mandatory when:

```text
complexity =
  entities
+ triads
+ 2 * routes
+ 2 * conflicting_sources
+ 3 * high_risk_role_swaps
```

and:

```text
complexity >= 12
```

The threshold is not proof of confusion. It is a practical guardrail for when
the agent should stop trusting direct reasoning and call the checker.

## Encoding Channels

Each triad has four independent channels:

```text
subject role/entity binding
relation mode
object role/entity binding
evidence support binding
```

The first wave core should not store whole patterns as rows. It should store
and score sparse bindings:

```text
subject_binding = bind(role(subject), bind(position(subject), permute(entity(subject), 17)))
object_binding  = bind(role(object), bind(position(object), permute(entity(object), 73)))
relation_mode   = mode(relation)
triad_mode      = compose(subject_binding, relation_mode, object_binding)
support_mode    = bind(triad_mode, evidence_ref)
```

The composite mode must change when subject and object are swapped.
That requires explicit positional modes. Plain multiplication of role/entity
bindings is not enough because it is commutative. V0 uses different positional
permutations for subject and object lanes.

V0 builds a source memory from `triads` and scores each `candidate_triads`
composite against that memory. A swapped candidate should have high token
overlap but low composite similarity.

## Core Size v2.2

Use fixed dimensions for the current recursive comb/search/agent-field/WAW
dataset-immunity, source-weighted search, auto-query, and learning negative-lane
verifier, plus route-balanced focus, coarse-to-fine trace, and polarization:

```text
wave_dim:       1024 lanes
active_triads:    32
active_entities:  16
active_roles:      8
active_relations: 16
```

Rationale:

- small enough for trivial local runtime;
- large enough to test role swaps and route conflicts;
- no dependency on huge cell banks before the eval proves value.

Memory target for the current core:

```text
wave vectors:      < 1 MB
scratch/trace:     < 1 MB
total hot state:   < 2 MB
```

This is intentionally below L2/L3 pressure. The public skill should prove the
gate before it grows into a larger NANDA runtime.

## Cells

For this skill, a "cell" is not a repeated 8 KB object yet. The first checker
uses logical cells:

```text
EntityCell     - one entity vector and aliases
RoleCell       - one role vector
RelationCell   - one relation vector
TriadCell      - one composed subject/relation/object binding
EvidenceCell   - one evidence binding
RouteCell      - ordered/typed group of triads
VerdictCell    - accumulated score and decision state
```

Counts in V0:

```text
EntityCell:    up to 16 target, 32 hard
RoleCell:       up to 8 target, 16 hard
RelationCell:  up to 16 target, 32 hard
TriadCell:     up to 32 target, 64 hard
EvidenceCell:  up to 32 target, 64 hard
RouteCell:      up to 4 target, 8 hard
VerdictCell:    1 per candidate answer
```

Only after `SPARSE-TRIAD-0` passes should these logical cells be packed into a
cache-local binary layout.

## Scores

The checker computes:

```text
local_binding_score
composite_triad_score
route_coherence_score
evidence_support_score
conflict_score
baseline_scores
```

V0 computes route coherence by aggregating triad waves per `group`.
A candidate group must match one source group as a whole. This catches
route-splice cases where every candidate triad is individually true, but the
candidate answer combines triads from different source groups.

Decision rule V0:

```text
PASS:
  composite_triad_score high
  route_coherence_score high
  conflict_score low
  evidence gaps absent

WATCH:
  missing evidence
  low confidence extraction
  task too large
  NANDA and baselines disagree

VETO:
  role swap detected
  route contradiction detected
  route-splice group detected
  route_coherence_score below threshold for multi-triad candidate group
  same evidence bound to incompatible triads
```

## Baselines

No NANDA result is interesting unless compared against:

```text
exact symbolic rule
token overlap
cosine/vector similarity
simple graph consistency rule
optional LLM judge later
```

The first claim of value requires cases where naive similarity stays high but
NANDA rejects the broken binding.

## Non-Expansion Rule

Do not increase cells, dimensions, or memory until a smaller eval fails for a
specific measured reason.

Allowed growth order:

```text
more test cases
better corruption set
better baselines
then larger wave_dim
then more triads
then packed cache-local cells
```

This prevents architecture drift.
