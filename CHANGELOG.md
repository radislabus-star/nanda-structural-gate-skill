# Changelog

## Unreleased

### Changed

- NANDA-6M lane preview/application now calls the packed hot-core
  `nanda_6m` lane compiler and applier instead of using JSON-derived lane
  deltas as the source of truth.
- `nanda bench6m` now includes a packed lane application kernel benchmark and
  `--mode lane`.
- `nanda bench6m` now includes `--mode lane-sweep` for batch packed lane arena
  application over support fields.
- `nanda bench6m` now includes `--mode aligned-lane-sweep` for direct
  field/lane windows without arena search.
- `nanda bench6m` now includes `--mode aligned-compile-sweep` for fused aligned
  lane compilation and application.
- NANDA-6M has typed packed peak decision and support-field builder contracts.
- `nanda bench6m` now includes `--mode support-build` and
  `--mode support-build-compile-sweep`.
- NANDA-6M support build now scores single packed triads directly without
  constructing temporary one-triad centroids.
- NANDA-6M support scoring reuses precomputed query energy inside support-field
  scans.
- NANDA-6M now has per-triad support score caches and support-field assembly
  from cached route/group dot scores.
- `nanda bench6m` now includes `--mode support-score-build` and
  `--mode support-score-build-compile-sweep`.
- NANDA-6M now has route/group support score bucket assembly for faster field
  construction from cached scores.
- `nanda bench6m` now includes `--mode support-bucket-build` and
  `--mode support-bucket-build-compile-sweep`.
- NANDA-6M now has a typed hot-cycle API that runs support scoring, route/group
  bucket assembly, batched support-field construction, lane compilation, and
  aligned lane sweep as one cache-resident core call.
- `nanda bench6m` now includes `--mode hot-cycle`.
- NANDA-6M now has a packed runtime contract around `PackedHotCore`: it
  validates active memory, centroid count, resident lanes, field requests, and
  hot workspace bytes before attaching the runtime.
- `nanda pack6m` and `nanda bench6m --mode hot-cycle` now report
  `runtime_contract`, including `PACKED_RUNTIME_READY`, `FOCUS_REQUIRED`,
  `SPLIT_REQUIRED`, `SPILL_REQUIRED`, and `WORKSPACE_TOO_SMALL`.
- NANDA-6M now fixes the active proof window at 15,000 triads with 64 default
  field requests. The broad triad arena still stores up to 65,536 records, but
  hot proof requires a focused 15k window.
- NANDA-6M core is now split into focused `wave` and `replay` submodules while
  preserving the public `nanda_6m::*` API.
- NANDA-6M support scoring now accumulates signed query projection first and
  applies triad strength once, reducing the 15k/64 hot-cycle release benchmark
  from about 105.6 ms/op to about 56.7 ms/op on the local T480 run.
- `nanda focus` / `nanda-focus` builds a physical route-balanced focused JSON
  packet with `--max-triads` defaulting to the 15,000-triad hot proof cap.
- `nanda proof` / `nanda-proof` runs the v25 proof-from-focus pipeline:
  dataset doctor, focused packet builder, NANDA-6M budget, packed bridge, and
  interference search in one report.
- v26 proof reports now expose top-level `proof_state`, `answer_ready`,
  `runtime_ready`, `hot_proof`, and `repair` fields so agents can distinguish
  "runtime/packed peak ready" from "safe to answer".
- v27 proof reports now expose top-level `reason_codes`,
  `proof_confidence`, and `proof_compare`, so WATCH/VETO states say whether the
  blocker is a thin field, reversed field, packed peak, focus shift, corpus
  warning, or runtime budget issue.
- `nanda proof --suite` / `nanda-proof --suite` now runs a proof regression
  corpus against expected proof states, top peaks, field states, and reason
  codes.
- v28 `nanda search` reports `resonant_field`, a physical field layer for
  phase lock, standing-wave reflection, route-boundary leakage, destructive
  locality, multiscale agreement, energy accounting, mode scan, temporal phase,
  coherence memory, and `WAW_RESONANCE` thresholding.
- v28 proof reason codes now surface resonance blockers such as
  `WAW_RESONANCE`, `RESONANCE_FIELD_DIFFUSE`,
  `RESONANCE_FIELD_LEAKING`, and `RESONANCE_REVERSED`.
- v29 feedback now records `resonance_memory`, a learned field-shape memory
  for accepted/rejected/WATCH peaks. `nanda-index` merges these forms and
  `nanda-search` softly replays them to reinforce known honest resonance forms
  or suppress known bad field shapes without treating memory as proof.
- v30 adds `nanda decode` / `nanda-decode`, the first LLMWave bridge. It runs
  the interference field and decodes ranked `next_structural_pattern`
  candidates instead of natural-language text.
- v31 adds recurrent decoding through `nanda decode --steps N`: selected
  patterns are fed back as query context and the field is re-run until the
  requested depth or `PATTERN_SATURATED`.
- v32 adds `nanda decode-eval` / `nanda-decode-eval` and
  `examples/decode-corpus.json` for decoder continuation regressions.
- v33 adds `nanda encode` / `nanda-encode`, the deterministic token/pattern
  encoder that turns raw text into a 1024-dimensional wave signature and
  optional query packet.
- v34 adds continuation training: `nanda-feedback` can now accept decode
  output, `nanda-index` merges `continuation_memory`, and `nanda-decode`
  reinforces accepted continuations or suppresses rejected ones.
- v35 adds a compact 32-byte pattern store for continuation memory.
- v36 replays the compact pattern store during decode ranking.
- v37 adds `nanda-pattern-capacity` for 1k/4k/16k/64k learned-pattern checks.
- v38 makes rejected continuations shortcut-specific negative lanes.
- v39 adds `nanda-llmwave`, a raw text -> encode -> decode -> feedback-preview
  mini-loop.
- v40 reports the NANDA-6M pattern runtime contract in budget/pack outputs.
- v41 adds `nanda-pattern-eval` and `examples/pattern-learning-corpus.json`
  to measure baseline vs trained decode learning effects.
- v42 adds beam decode: `--beam-width N` keeps ranked structural trajectories.
- v43 extends `nanda-decode-eval` with beam trajectory checks.
- v44 exposes pre-ranking pattern replay under `early_pattern_replay`.
- v45 adds opt-in field-state-aware decode scoring with `--adaptive-scoring`.
- v46 adds `nanda-pattern-bank` for standalone learned continuation banks.
- v47 adds an HRR binding sandbox inside `nanda-llmwave`: role/filler lanes are
  bound and unbound back to subject/object vectors as a measurable probe.
- v48 turns `nanda-pattern-bank` into a cleanup-memory contract for noisy
  decoded structural patterns.
- v49 adds an attractor-style `energy_trace` to recurrent decode.
- v50 adds a superposition capacity report to the LLMWave loop.
- v51 adds a shortcut-specific anti-wave audit for rejected continuation lanes.
- v52 upgrades `nanda-llmwave` to a read/write/retrieve loop that reports HRR
  binding, cleanup, attractor energy, capacity, anti-wave locality, decode, and
  feedback preview in one packet.
- v53 adds `nanda-llmwave-eval` and `examples/llmwave-corpus.json` to verify
  the full LLMWave proof packet.
- v54 reports packed HRR lanes as a fixed 64-byte hot-lane contract.
- v55 adds cleanup dictionary thresholds.
- v56 adds anti-wave locality checks.
- v57 adds direct-table vs packed-wave capacity baseline estimates.
- v58 collapses LLMWave reports into a packed hot-cycle readiness state.
- v59 adds the LLMWave proof-summary contract.
- v60 exposes a public demo packet with a safe claim and proof state.
- v61 adds `nanda-demo`, a human/agent-facing weak-spot surface over the v60
  proof packet, plus `examples/demo-corpus.json` with ready, anti-wave, and
  review cases.
- Core version is now `sparse-triad-v6.0-llmwave-proof`.
- Cargo package version is now `6.0.0`.
- Added an explicit heavy 16k finance fixture generator and
  `scripts/test-heavy-16k.sh` to verify route-balanced corpus loading, full
  16,384-triad search, packed `FOCUS_REQUIRED`, and focused proof behavior
  without committing a multi-megabyte JSON fixture.
- NANDA-6M version is now `nanda-6m-v40-llmwave-pattern-runtime`.

## v3.3.0 - 2026-06-18

Modular router and code-map planning release.

### Added

- `nanda map-code` / `nanda-map-code` refactor-planning command.
- `dogfood --refactor-plan` to attach code-boundary recommendations to the
  structural dogfood verdict.
- Focused Rust modules for `model`, `io`, `map_gate`, `search`, `feedback`,
  `dataset_doctor`, `eval`, `report`, and `commands/dogfood`.

### Changed

- `src/main.rs` is now a compact CLI router instead of the implementation
  body.
- `map-code` now treats `main/run/run_check` as `cli-router`, filters that
  router from `next_refactors`, and ignores nested test functions.
- Runtime install and local tests cover `nanda-map-code` and dogfood refactor
  planning.
- Clippy is clean with `cargo clippy --all-targets --all-features -- -D warnings`.
- Core version is now `sparse-triad-v3.3-modular-router`.
- Engine id is now `nanda-check sparse-triad-v3.3-rust`.
- Cargo package version is now `3.3.0`.

## v3.2.0 - 2026-06-18

Canonical aliases release.

### Added

- `aliases` packet field for explicit high-confidence canonicalization.
- `nanda aliases` / `nanda-aliases` diagnostic command.
- Canonicalization trace in gate reports, map/search/budget/pack6m output, and
  trace files.
- Alias fixtures for naming drift PASS, real post-alias conflict VETO, and
  ambiguous alias WATCH.

### Changed

- `subject`, `object`, `route`, and `group` are canonicalized before structural
  checks when explicit aliases are present.
- `search --query-file` inherits aliases from the memory packet when the query
  packet has none.
- `issued_by` is treated as a functional relation.
- Core version is now `sparse-triad-v3.2-canonical-aliases`.
- Engine id is now `nanda-check sparse-triad-v3.2-rust`.
- Cargo package version is now `3.2.0`.

## v3.1.0 - 2026-06-18

NANDA-6M hot benchmark and full-model contract release.

### Added

- `nanda bench6m` / `nanda-bench6m` hot-core microbenchmark.
- Replay-firewall benchmark for `nanda_6m::evaluate_replay` without JSON,
  file I/O, process startup, or report serialization.
- Packed projection benchmark for 1024-dimensional in-memory projection,
  centroid construction, and centroid scoring.
- Runtime, Linux install, Windows install, README, and skill documentation for
  `nanda-bench6m`.
- `ARCHITECTURE_NANDA_6M.md` section defining what a "full NANDA-6M model"
  means in this project.

### Changed

- Cargo package version is now `3.1.0`.
- CI no longer builds or installs Windows release wrappers by default. Windows
  packaging remains manual only.

## v3.0.1 - 2026-06-18

Windows CI and wrapper fix.

### Added

- GitHub Actions `windows-latest` job with Rust tests, release build,
  PowerShell install smoke, `nanda-doctor.cmd`, `nanda-pack6m.cmd`, and
  `nanda-self-check.cmd`.

### Fixed

- `scripts/install-windows.ps1` now generates Windows `.cmd` wrappers for
  `nanda-budget.cmd` and `nanda-pack6m.cmd`.

### Changed

- Cargo package version is now `3.0.1`.

## v3.0.0 - 2026-06-18

Hot replay-core boundary release.

### Added

- Typed replay structs/enums in `src/nanda_6m.rs`: raw peak state, replay
  compute state, replay field state, replay stability state, replay verdict,
  replay action, replay touch, replay decision input, and replay decision.
- `nanda_6m::evaluate_replay`, a pure hot-compatible replay firewall function
  with no JSON, strings, maps, or serde in the decision path.
- JSON bridge parity: `packed_replay_decision` is now built from typed
  `evaluate_replay` output and reports `core="nanda_6m::evaluate_replay"` plus
  `hot_compatible=true`.
- Hot-module unit tests for no replay evidence, stable-with-replay,
  rescued-thin-field, destabilized-field, and full-touch-required cases.

### Changed

- Cargo package version is now `3.0.0`.
- Core version is now `sparse-triad-v3.0-hot-replay-core`.
- Engine id is now `nanda-check sparse-triad-v3.0-rust`.
- NANDA-6M version is now `nanda-6m-v4-hot-replay-core`.

## v2.9.0 - 2026-06-18

Replay-firewall release.

### Added

- `packed_replay_decision` in `nanda pack6m` output.
- Replay stability verdicts:
  `STABLE_WITH_REPLAY`, `REPLAY_RESCUED_THIN_FIELD`,
  `REPLAY_DESTABILIZED_FIELD`, `REPLAY_TOO_STRONG_REQUIRED`, and
  `NO_REPLAY_EVIDENCE`.
- Replay firewall rules that allow replay to shape or rescue the packed field
  while still blocking direct `safe_to_answer=true`.
- `examples/triad-packet.pack6m-replay-waw.json`, a WAW fixture where the raw
  packed field is thin and soft replay rescues it into review-ready state.
- Text and Markdown `pack6m` output now show replay decision/action.

### Changed

- Cargo package version is now `2.9.0`.
- Core version is now `sparse-triad-v2.9-replay-firewall`.
- Engine id is now `nanda-check sparse-triad-v2.9-rust`.

## v2.8.0 - 2026-06-18

Packed replay release.

### Added

- `packed_lane_replay.touch_policy` for observer-to-compute replay semantics.
- `packed_lane_replay.stability_sweep` with observer, soft-touch,
  medium-touch, and full-touch strengths.
- `packed_lane_replay.stability_state` to distinguish no replay, soft-stable
  replay, full-touch-only replay, weak constructive replay, and destabilizing
  replay.
- `packed_lane_replay.computational_effect` to report when matched feedback
  lanes are ready to shape the packed field without granting answer
  permission.
- Text and Markdown `pack6m` output now show replay stability and compute
  readiness.

### Changed

- Cargo package version is now `2.8.0`.
- Core version is now `sparse-triad-v2.8-packed-replay`.
- Engine id is now `nanda-check sparse-triad-v2.8-rust`.

## v2.7.0 - 2026-06-17

Hierarchical-gate release.

### Added

- `nanda hgate` and `nanda-hgate` wrapper for large structural packets.
- Hierarchical gate output with `global`, `branches`, and
  `hierarchical_decision`.
- `STRUCTURALLY_ACCEPTED` action for the important case where the global packet
  is `WATCH` only because of size/field dilution, while every linked local
  branch is `PASS`.
- `REPAIR_REQUIRED` action when the global map has `foreign_pull`, conflicts,
  or any local branch is `VETO`.
- Size-only hierarchical fixture covering a large global graph with 17/17
  local branches passing.
- Linux and Windows CI smoke coverage for `nanda hgate`.

### Changed

- Cargo package version is now `2.7.0`.
- Core version is now `sparse-triad-v2.7-hierarchical-gate`.
- Engine id is now `nanda-check sparse-triad-v2.7-rust`.

## v2.6.0 - 2026-06-17

Feedback-memory release.

### Added

- `positive_shortcuts` in triad packets for accepted route reinforcement.
- `nanda feedback --decision accept` now emits a reusable positive shortcut
  with `reinforce_peak`, `reinforce_route`, `reinforce_group`, query terms,
  support terms, and `accepted_count`.
- `nanda index` now ingests feedback-memory JSON files with both negative and
  positive lanes, merges repeated accepts, and accumulates `accepted_count`.
- `constructive_interference` in `nanda search` output. It reports applied
  positive-lane boosts, effective boost, match ratios, support ratios, and
  learned accepted counts.
- Positive-lane fixture and Linux/Windows CI smoke coverage.

### Changed

- Cargo package version is now `2.6.0`.
- Core version is now `sparse-triad-v2.6-feedback-memory`.
- Engine id is now `nanda-check sparse-triad-v2.6-rust`.

## v2.5.1 - 2026-06-17

Probe-suite completion release.

### Added

- `nanda probe --suite` for regression suites of probe decisions.
- Probe corpus fixture covering `SHIFTED_TO_REVIEW`, `UNCHANGED`, grouped
  negative-lane suppression, and external negative-lane input.
- Linux and Windows CI smoke coverage for `nanda probe --suite`.

### Changed

- Cargo package version is now `2.5.1`.
- Core version remains `sparse-triad-v2.5-probe-report`.

## v2.5.0 - 2026-06-17

Probe-report release.

### Added

- `nanda probe` and `nanda-probe` wrapper for before/after search checks.
- `nanda probe --suite` for regression suites of probe decisions.
- Probe output compares plain search with negative-lane search and reports
  `plain`, `negative`, `delta`, `decision`, and `recommended_action`.
- `SHIFTED_TO_REVIEW` decision for cases where destructive interference moves
  the top peak but the new field is still not safe to answer from.
- Probe corpus fixture covering `SHIFTED_TO_REVIEW`, `UNCHANGED`, grouped
  negative-lane suppression, and external negative-lane input.
- Linux and Windows CI smoke coverage for probe reports.

### Changed

- Core version is now `sparse-triad-v2.5-probe-report`.
- Cargo package version is now `2.5.0`.

## v2.4.0 - 2026-06-17

Local destructive-interference release.

### Added

- Group-aware destructive interference. Negative lanes now match peaks through
  peak name, route, group, and `route:group` composites, so a route-level lane
  can suppress a grouped peak such as `w-field-debt:canon_current`.
- Shortcut-local negative lanes. Reject feedback now records
  `suppress_route`, `suppress_group`, `prefer_route`, `prefer_group`, and
  `support_terms` so suppression targets the rejected reading shape instead of
  blindly killing every future peak with a similar name.
- Top-level search contract: `verdict`, `field_state`, `safe_to_answer`, and
  `top_peak` are now exposed directly on `nanda search` output.

### Changed

- Destructive-interference output now includes both `suppress_peak` and the
  backwards-compatible `suppressed_peak`.
- Dataset doctor now reports `large_unbalanced_corpus` for oversized
  route-heavy corpora and can emit `large_but_route_balanced_focus` as a
  notice instead of treating every large packet as the same failure.
- Core version is now `sparse-triad-v2.4-local-negative-lanes`.
- Cargo package version is now `2.4.0`.

## v2.3.0 - 2026-06-17

Field-state-machine release.

### Added

- `field_state_machine` in `nanda search` output. It converts measured field
  signals into explicit agent states such as `FIELD_FOCUSED`,
  `FIELD_CONTESTED`, `FIELD_REVERSED`, `FIELD_NOISY`, and `FIELD_THIN`.
- State-machine actions such as `ANSWER_WITH_SUPPORT`, `SPLIT_OR_QUERY`,
  `FOCUS_CORPUS`, and `STOP_REPAIR_POLARITY`.
- Regression checks proving that route traps, noisy fields, route-balanced
  low-margin fields, and reversed polarity fields land in different states.

### Changed

- Core version is now `sparse-triad-v2.3-field-state-machine`.
- Cargo package version is now `2.3.0`.

## v2.2.0 - 2026-06-17

Polarity-gate release.

### Added

- Polarity-aware peak scoring: reversed role-direction polarity applies a
  `polarization_penalty` to the peak score.
- `peak_decision.state = POLARITY_REVERSED` with `safe_to_answer=false` when
  the top peak is structurally reversed relative to the query.
- `field_interpretation.state = polarity_reversed` for reversed top peaks.
- Reversed-polarity stop fixture and CI smoke coverage.

### Changed

- Core version is now `sparse-triad-v2.2-polarity-gate`.
- Cargo package version is now `2.2.0`.

## v2.1.0 - 2026-06-17

Polarized-field release. This folds the v1.9 route-balanced and v2.0
coarse-to-fine milestones into one published core.

### Added

- `nanda search --route-cap` and `--route-triad-cap` route-balanced focus for
  large/noisy memory packets.
- `route_balanced_focus` output showing original memory size, focused memory
  size, per-route selected triads, and why focus was applied.
- `coarse_to_fine` output: first pick the coarse route peak, then expose the
  local supporting path inside that route.
- Polarization lane in triad wave encoding so role direction contributes to
  resonance, not only to post-hoc explanation.
- `polarization` output per peak and `polarity` per support/anti triad.
- Route-balanced and polarization fixtures with local and CI smoke coverage.

### Changed

- Core version is now `sparse-triad-v2.1-polarized-field`.
- Cargo package version is now `2.1.0`.

## v1.8.0 - 2026-06-17

Learning-lanes release. This is the first release after the v1.6/v1.7 scoring
steps; no separate Windows release was cut for those internal milestones.

### Added

- Repeated reject feedback now learns stronger negative lanes. `nanda index`
  merges duplicate `negative_shortcuts`, accumulates `observations` and
  `rejected_count`, and future search reports the learned `effective_penalty`.
- Local regression coverage proving that two rejects produce a stronger
  destructive-interference penalty than one reject.

### Changed

- Core version is now `sparse-triad-v1.8-learning-lanes`.
- Cargo package version is now `1.8.0`.

## v1.7.0 - 2026-06-17

Auto-query release.

### Added

- `nanda search` now builds lightweight auto query triads from `--query` or
  packet `query` when no `candidate_triads` are provided.
- Search output now exposes `query.source`, with values such as
  `candidate_triads`, `auto_query_triads`, or `empty`.
- Auto-query fixture proving a text-only query can activate a specific route
  without hand-authored candidate triads.

## v1.6.0 - 2026-06-17

Source-weighting release.

### Added

- Source/confidence weighting in interference search. Triad confidence is
  multiplied by evidence authority so current/canonical evidence pulls harder
  than historical/archive/noise evidence.
- `source_weighting` metadata in search output.
- `source_weight` on supporting and anti triads for explanation.
- Source-weighting fixture proving `current-frontier` beats archive noise.

## v1.4.0 - 2026-06-17

Negative-lanes release.

### Added

- `negative_shortcuts` in triad packets for suppressing known false peaks.
- Destructive-interference scoring in `nanda search`: matching negative lanes
  suppress rejected peaks and lightly boost preferred alternatives.
- `destructive_interference` output with applied suppressions, penalty, match
  ratio, preferred peak, and reason.
- `nanda feedback --decision reject` now emits a reusable negative shortcut.
- `nanda index` can ingest feedback-memory JSON files and carry negative lanes
  into indexed memory packets.
- Negative shortcut fixtures proving `customs` wins without a lane and
  `certification` wins after the lane is applied.

### Changed

- Core version is now `sparse-triad-v1.4-negative-lanes`.

## v1.3.0 - 2026-06-17

Dataset-immunity release.

### Added

- `nanda dataset-doctor` and `nanda-dataset-doctor` wrapper for corpus-quality
  checks before search.
- Corpus diagnostics for route imbalance, hub dominance, duplicate CURRENT
  facts, weak text-only queries, and oversized direct-search packets.
- `field_interpretation.corpus` in search output so agents can see corpus noise
  warnings next to the peak.
- Mini dataset-noise fixture that models the large Wave Atlas failure mode.
- Linux and Windows CI coverage for dataset-doctor WATCH behavior.

### Changed

- Core version is now `sparse-triad-v1.3-dataset-immunity`.

## v1.2.0 - 2026-06-17

WAW benchmark release.

### Added

- `nanda waw --suite examples/waw-corpus.json` for hard route-trap benchmark
  cases where the structural interference peak must beat the lexical baseline.
- WAW corpus with business, code/runtime, and document-ownership traps.
- `nanda-waw` shell wrapper and Windows `.cmd` wrapper generation.
- CI coverage for the WAW corpus on Linux and Windows.

### Changed

- Core version is now `sparse-triad-v1.2-waw-benchmark`.
- Local tests now require `structural_wins`, `lexical_traps`, and
  `explainable_drifts` across the WAW corpus.

## v1.1.0 - 2026-06-17

Agent-field strengthening release.

### Added

- Eval corpus loading via `nanda eval --suite examples/eval-corpus.json`.
- JSONL agent API via `nanda serve`, supporting `doctor`, `check`, and `search`
  requests without restarting the process for every call.
- Richer `nanda search` output with `field_interpretation`, including field
  stability, lexical-trap detection, centroid drift, and nearest foreign pull.
- `nanda-serve` shell wrapper and Windows `.cmd` wrapper generation.

### Changed

- Core version is now `sparse-triad-v1.1-agent-field`.
- Local tests include Rust unit tests, eval corpus regression, JSONL serve
  smoke, and field-interpretation assertions.

## v1.0.1 - 2026-06-17

Windows compatibility release.

### Added

- PowerShell installer: `scripts/install-windows.ps1`.
- Windows `.cmd` wrappers for `nanda`, `nanda-check`, `nanda-search`,
  `nanda-doctor`, and the rest of the command surface.
- Windows CI job that builds, tests, runs `doctor`, runs `eval`, and verifies
  the PowerShell installer.

### Fixed

- `nanda eval --case` now parses Windows drive-letter paths such as
  `C:\repo\case.json:certification:FOCUSED` by splitting the case spec from
  the right.

## v1.0.0 - 2026-06-17

Initial public release of NANDA Structural Gate.

### Added

- Rust CLI core with `PASS`, `WATCH`, and `VETO` structural verdicts.
- Codex skill wrapper in `nanda-structural-gate/`.
- Structural map, linked-group split, recursive comb, dogfood self-check.
- Interference retrieval with route/group peaks, support triads, anti-triads,
  missing edges, answer projection, lexical baseline, and `peak_decision`.
- Memory index, arrow-text extraction, feedback traces, eval suite, and release
  doctor smoke test.
- Examples for role swaps, route splices, missing evidence, code-flow checks,
  invariant drift, linked-group split, and interference route traps.
- Local install, test, benchmark, and runtime sync scripts.

### Validation

- `scripts/test-local.sh`: pass
- `scripts/test-edge-cases.sh`: pass
- `scripts/benchmark-v0.sh`: pass
- `nanda-doctor`: healthy
- `nanda-eval`: 2/2 route-trap/noisy cases
- `nanda-dogfood .`: `SAFE_TO_EDIT`, 7/7 local branches pass
