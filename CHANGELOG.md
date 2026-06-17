# Changelog

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
