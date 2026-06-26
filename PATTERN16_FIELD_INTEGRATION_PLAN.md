# Pattern16 Field Integration Plan

Status: Pattern16 structural capacity now reuses the existing shared field
physics for admission readout.

## Current State

- `structural-capacity` is fixed to 1024 Pattern16 macro-cells.
- `--noise-profile skill-admission` raises the stress run to at least 8 seeds
  and 16 foreign-edge noisy additions per positive query.
- The report exposes `lens_admission`.
- `lens_admission` calls the existing `field_core` path:
  - `field_core::lens::apply_lens_chain`
  - `field_core::anti_wave::apply_anti_wave`
  - `field_core::pass::run_field_pass`
- The readout confirms the Pattern16 peak while preserving the claim boundary:
  `field_pass_safe_to_answer=false`.
- `lens-scan` and `mature-anti-wave` now expose `field_core_admission`, so the
  downstream lens/anti-wave pipeline also reports through `FieldPassReport`.
- `nanda-field-audit` exposes `sole_engine_contract`, which registers Pattern16,
  lens scan, and mature anti-wave as consumers of the same field physics.

## Integration Rules

1. Do not create a second lens system inside Pattern16.
2. Do not create a second anti-wave system inside Pattern16.
3. Pattern16 admission may add stress/noise policy, but lens and anti-wave
   operations must come from `field_core`.
4. `field_pass_verdict=WATCH` is acceptable when it means the readout is
   claim-boundary-preserving rather than answer-permissive.
5. No 256/512/Pattern4 fallback may return through integration work.
6. Do not claim broad LLM, global nonlinear memory, real-corpus capacity, or
   hardware PMU residency from this gate.

## Next Integration Phases

### Phase 1: Shared Admission Type

Move the local Pattern16 admission shape toward a shared `field_core`
admission readout type.

Target:

```text
field_core::admission::FieldAdmissionReadout
```

It should carry:

- field core version;
- field pass version;
- lens chain;
- anti-wave lanes;
- peak target;
- peak state;
- coherence state;
- safe-to-answer flag;
- claim-boundary preservation;
- admission verdict.

### Phase 2: Lens Scan Bridge

Status: implemented as `LensScanReport.field_core_admission`.

Bridge `llmwave_big::lens_scan` to the shared field pass without duplicating
fixtures.

Target:

```text
structural_capacity.lens_admission
  -> optional lens_scan_compat
  -> role/evidence/temporal/causal/answer agreement
```

The bridge must show whether the Pattern16 peak stays the same under the lens
scan, not whether a natural-language answer is allowed.

### Phase 3: Mature Anti-Wave Bridge

Status: implemented as `MatureAntiWaveReport.field_core_admission`.

Bridge `llmwave_big::mature_anti_wave` to the shared field pass.

Target:

```text
blocking lenses -> local anti-wave lanes -> false-peak suppression
```

The bridge must prove that anti-wave lanes suppress false readings without
deleting the true Pattern16 peak.

### Phase 4: Real Extraction Admission

Connect live extracted packets to Pattern16:

```text
text/code/contract
  -> extracted Pattern16 candidate
  -> structural-capacity admission
  -> field-core lens readout
  -> route gate
```

This phase decides whether the skill can use Pattern16 as an admission core for
real tasks rather than only synthetic patterns.

### Phase 5: Held-Out Pattern16 Suite

Build a held-out suite with:

- role swaps;
- route splices;
- evidence gaps;
- stale/current swaps;
- same-clause multi-effect traps;
- code owner/adapter leaks;
- namespace collisions.

Acceptance:

```text
false_accept_rate = 0
false_negative_rate = 0
single_peak_under_noise = true
lens_admission.accepted_for_skill_admission = true
claim_boundary_preserved = true
```

### Phase 6: Skill Workflow Cutover

Use Pattern16 admission in the actual skill workflow only when:

```text
extractor quality is measured
held-out suite passes
route gate remains non-permissive
field_core lens/anti-wave owners stay shared
```

Until then, Pattern16 is a structural core admission result, not a full
end-to-end skill proof.

## Immediate Fix Queue

Done in the Pattern16 admission integration pass:

1. Replace docs that imply Pattern16 owns a separate lens model.
2. Add tests that `lens_admission` uses `field_core` versions.
3. Add tests that `field_pass_safe_to_answer=false` remains true under Pattern16
   admission.
4. Add a compact JSON example for `--noise-profile skill-admission`.

Remaining:

1. Consider extracting the local Pattern16 admission struct into `field_core`
   once a second caller needs the same exact readout shape.
2. Use `sole_engine_contract` as the acceptance registry before adding any new
   large field pipeline.
