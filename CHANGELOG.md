# Changelog

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

