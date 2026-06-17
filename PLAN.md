# Plan

## Current State

The project is a clean public skill source tree with a Rust V0 checker and
stable script wrappers for Codex/runtime use.

Runtime behavior is intentionally conservative:

```text
PASS | WATCH | VETO
engine: nanda-check sparse-triad-v1.3-rust
core_version: sparse-triad-v1.3-dataset-immunity
```

## Build Order

1. Keep the skill lightweight and auto-triggerable.
2. Define the complexity threshold that makes the gate mandatory.
3. Freeze the V0 input/output/cell architecture.
4. Define triad input and verdict output contracts.
5. Implement the sparse-triad V0 checker. Done in Rust.
6. Wire the checker into `nanda-check`. Done through stable wrappers.
7. Add baselines before interpreting results. Done for exact/reversed/token
   overlap and route-splice false accepts.
8. Sync the source skill into `.codex/skills`. Done by `scripts/install-local.sh`.
9. Split oversized worksheets by group or route. Done with `nanda-split` and
   `nanda-split-md`.
10. Add path normalization for code checks. Done with `--normalize-paths`.
11. Add structural interference map. Done with `nanda-map` and
    `structural_map`.
12. Add centroid/foreign-pull map. Done in `v0.3-centroid`.
13. Add linked-group split. Done with `nanda-split --by linked-group` and
    `nanda-split-md --by linked-group`.
14. Add JSON-first split for machine flows. Done with `nanda split`.
15. Add recursive comb tree. Done with `nanda comb --depth`.
16. Add repository dogfood gate. Done with `nanda dogfood .`.
17. Add interference retrieval. Done with `nanda search`.
18. Add reusable memory index. Done with `nanda index` and
    `nanda search --query-file`.
19. Add simple triad extraction. Done with `nanda extract`.
20. Add feedback memory packets. Done with `nanda feedback`.
21. Add regression eval for peak/state behavior. Done with `nanda eval`.
22. Add self-contained release doctor. Done with `nanda doctor`.
23. Add file-backed eval suites. Done with `nanda eval --suite`.
24. Add JSONL agent API. Done with `nanda serve`.
25. Add field interpretation for interference peaks. Done in
    `field_interpretation`.
26. Add WAW benchmark for lexical-trap wins. Done with `nanda waw`.
27. Add dataset-quality gate before search. Done with `nanda dataset-doctor`.

## Engineering Constraints

- Keep runtime calls cheap.
- Keep the shipped runtime core in Rust.
- Keep noisy traces in local files, not chat.
- Treat every unverified output as `WATCH`.
- Keep source project and runtime skill separate.
- Do not increase cells, dimensions, or memory before the small eval proves why.

## First Experiment Shape

```text
correct:   subject=A relation=supplies object=B
corrupted: subject=B relation=supplies object=A
```

NANDA only matters if the composite score rejects the corrupted structure when
token overlap and naive similarity remain high.

Status: the repository has one PASS example and one role-swap VETO example.
It also has a route-splice VETO example where exact candidate triads are stable
individually, but route coherence rejects the combined group.
The benchmark records exact-baseline false accepts for those splice cases.
JSON packets are the preferred machine-flow format. Markdown triad tables can
still be packed into JSON with `nanda-pack-from-md` for live agent use.
For normal agent work, use `nanda-init-md` and `nanda-gate-md` so the source
artifact remains human-readable.
For code work, use the `code` worksheet template to check source/runtime/CLI
chains before finalizing architecture-sensitive changes.
Use `skill` and `project` templates for Codex skill sync and repository
readiness summaries.
Use `nanda-self-check` to verify the NANDA gate contour before relying on it in
a long task.
Use `nanda-dogfood .` to verify a repository against its own self-packet before
architecture-sensitive edits. Its compact agent decision is the fast go/no-go
surface; the full comb JSON remains the audit trail.
Use `nanda-report` as an agent decision packet, not primarily as a human report.
Use `nanda-split --by group|route|linked-group` for JSON packet splits when the
graph exceeds route/entity limits.
Use `nanda-split-md --by linked-group` for manual Markdown worksheets.
Use `linked-group` when source and candidate groups have different names; it
uses the map to pair source/candidate route worksheets.
Use `--normalize-paths` for code worksheets when file paths should collapse into
module-like entities.
Use `nanda-map` before expanding public API or docs, because this is now the
core surface that higher layers should build on.
Recommended acceptance workflow is now fixed: global map first, global gate as
size/stop signal, linked-group split, then route-level gate per paired file.
For machine workflows, prefer `nanda comb --depth 2`; it records topology,
root map/gate, linked branches, and invariant drift checks in one JSON packet.
For retrieval workflows, use `nanda search`: memory triads plus partial query
triads produce top-k structural peaks. The first target experiment is a route
search where the correct connected route beats similar single facts from a
foreign route.
Read `field_interpretation` in search output before trusting the peak. It
labels stable, thin, contested, and lexical-trap cases and explains centroid
drift from the nearest competing peak.
For indexed retrieval workflows, use `nanda index` to build a reusable memory
packet, then query it with `nanda search --query-file`.
For extraction workflows, use `nanda extract` on arrow-text notes before
indexing or searching.
For regression workflows, keep suites in JSON and run
`nanda eval --suite examples/eval-corpus.json`.
For WAW workflows, keep trap suites in JSON and run
`nanda waw --suite examples/waw-corpus.json`; each case must show a structural
win over lexical baseline plus explainable centroid drift.
For large-corpus workflows, run `nanda dataset-doctor` before search. A WATCH
means the corpus should be route-balanced, deduplicated, or queried with
explicit candidate triads before trusting peaks.
For agent/runtime integration, prefer newline-delimited JSON through
`nanda serve` instead of shelling one command per small check.

## Trigger Model

The agent should estimate:

```text
complexity =
  entities
+ triads
+ 2 * routes
+ 2 * conflicting_sources
+ 3 * high_risk_role_swaps
```

The first threshold is `complexity >= 12`.

Examples:

- one simple relation: no gate;
- contract with supplier/buyer/goods/payment/delivery/certification: gate;
- two plausible answer versions with swapped roles: gate;
- multiple documents with inconsistent party names: gate.
