# Changelog

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
