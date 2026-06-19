# LLMWave Field + Lens Contract

Status: v80 implementation contract.
Updated: 2026-06-19.

LLMWave is not a prose generator yet. It is the next layer above the NANDA
interference field: a compact wave/VSA memory that stores structural patterns
in superposition and reads them through explicit lenses.

```text
prefix / context
  -> field excitation
  -> selected lens
  -> stable candidate peaks
  -> cleanup / polarity / energy checks
  -> top-k continuation candidates
  -> proof state
```

## Definition

LLMWave is:

- a field over encoded text and structural patterns;
- a packed pattern memory with cleanup and replay;
- a lens readout layer over the same field;
- a proof contract that says whether the readout is usable.

LLMWave is not:

- a JSON gate renamed as a language model;
- a table of next n-grams;
- a RAG wrapper;
- a natural-language claim without readout and proof state.

## Field

The field owns the shared state:

- encoded query/prefix triads;
- memory/source triads;
- decoded structural continuations;
- compact pattern store;
- HRR binding probes;
- attractor/energy trace;
- superposition/capacity state;
- anti-wave and cleanup reports;
- hot-cycle readiness.

The field may contain many possible continuations in superposition. It does
not decide by itself which view is being read.

## Lenses

A lens is a readout/projection over the field. The same field can be read
through different lenses.

Before a lens reads the field, LLMWave emits a repeatable `field_snapshot`:

- `snapshot_id`;
- token and pattern counts;
- energy, top score, second score, and margin;
- top peak and top pattern;
- anti-wave state.

The snapshot is cold metadata. It is meant for comparison, regression checks,
and explaining how a lens changed the visible field.

### Pattern Lens

Purpose: choose the next structural pattern.

Input:

```text
text prefix or structural query
```

Output:

```text
top-k subject -> relation -> object continuations
```

Trust:

- requires a decoded top pattern;
- requires cleanup not to be ambiguous;
- requires the proof summary to be ready or explicitly WATCH.

### Polarity Lens

Purpose: read direction and negation.

It must distinguish:

```text
bank -> gives credit -> company
company -> gives credit -> bank
allowed
not allowed
```

Output states:

```text
ALIGNED
DIRECTIONAL
REVERSED
NEGATED
ROLE_SWAPPED
DIRECTION_AMBIGUOUS
```

`DIRECTIONAL` covers role paths such as
`document->requires->evidence`.

Trust:

- `REVERSED`, `NEGATED`, or `ROLE_SWAPPED` cannot be accepted as a clean
  continuation without explicit proof context.

### Cleanup Lens

Purpose: map a raw decoded peak to known clean patterns.

Output states:

```text
EXACT
NEAR
AMBIGUOUS
EMPTY
```

Trust:

- `EXACT` and strong `NEAR` can support a readout;
- `AMBIGUOUS` is WATCH;
- `EMPTY` means the field found no cleanup anchor.

### Token Lens

Purpose: read the next token or short phrase from the same field.

Input:

```text
text prefix
```

Output:

```text
top-k next tokens and phrases
```

The first implementation uses:

- `TokenPatternRecord` built from known triads;
- deterministic token waves;
- relative position phase for the last prefix tokens;
- next-token resonance over token-pattern records;
- token cleanup dictionary;
- shortcut-specific token anti-wave;
- suffix/frequency baseline comparison.

Trust:

- `TOKEN_LENS_READY` means the top token has cleanup support and enough margin;
- `TOKEN_LENS_CONTESTED` means the top token is plausible but too close to a
  rival;
- `TOKEN_LENS_REVIEW` means token readout exists but full structural proof is
  not ready;
- anti-wave may suppress one false prefix+next shape without killing the token
  topic.

### Convex Lens

Purpose: gather aligned weak pattern waves into a route basin.

Input:

```text
decoded field candidates
```

Output:

```text
top basin, gathered score, support count, gain, and supporting patterns
```

Trust:

- `CONVEX_LENS_READY` means one basin dominates and has multi-pattern support;
- `CONVEX_LENS_REVIEW` means a basin exists but the margin or support is too
  thin;
- the lens answers "what peak forms if aligned signals are gathered?"

### Concave Lens

Purpose: separate a mixed or contested peak into rival branches.

Output:

```text
branch list, score separation, and competing branch count
```

Trust:

- `CONCAVE_LENS_SPLIT` means multiple branches remain close enough to inspect;
- `CONCAVE_LENS_SINGLE` means the field does not currently need spreading;
- this lens is a review aid, not an answer permission by itself.

### Prism Lens

Purpose: explain one visible peak by its spectral structural contributions.

Output dimensions:

- route;
- relation;
- role path;
- polarity;
- anti-wave state.

Trust:

- `PRISM_LENS_READY` means the peak has an explanation surface;
- it does not prove truth; it shows why the field made the peak visible.

### Role Lens

Purpose: read actor/action/target binding from the field.

Output:

```text
actor, action, target, role path, polarity, role-swap risk
```

Trust:

- `ROLE_LENS_READY` means the top binding is not reversed and no swap risk is
  visible in the inspected candidates;
- `ROLE_LENS_SWAP_RISK` is a stop signal for role-sensitive answers;
- this lens is the first semantic readout for "who does what to whom?"

### Temporal Lens

Purpose: read recurrent decode as order/sequence flow.

Output:

```text
steps, top pattern per step, route jumps, standing pattern flag
```

Trust:

- `TEMPORAL_LENS_ORDERED` means the recurrent route stayed ordered;
- `TEMPORAL_LENS_ROUTE_JUMP` means the sequence crossed routes and needs
  review;
- `TEMPORAL_LENS_STANDING` means a repeated pattern formed a standing wave.

### Evidence Lens

Purpose: separate evidence-backed peaks from plausible but unsupported peaks.

Output:

```text
triad evidence binding, missing count, conflict list
```

Trust:

- `EVIDENCE_LENS_READY` means inspected peaks are evidence-bound and conflict
  free;
- `EVIDENCE_LENS_PARTIAL` can support review, not final proof;
- `EVIDENCE_LENS_CONFLICT` and `EVIDENCE_LENS_TOP_MISSING` are unresolved.

### Energy Lens

Purpose: read basin stability and perturbation risk.

Output:

```text
final energy, margin, route jumps, dropping trend, contested state
```

Trust:

- `ENERGY_LENS_STABLE` means the attractor and margin support the peak;
- `ENERGY_LENS_CONTESTED` means the field is alive but close alternatives
  remain;
- `ENERGY_LENS_UNSTABLE` means route jumps or dropping energy occurred.

### Anti Lens

Purpose: explain destructive interference.

Output:

```text
negative records, suppressions, reinforcements, top-after, changed-field flag
```

Trust:

- `ANTI_LENS_SUPPRESSED_SHORTCUT` means a shortcut-specific reject lane fired
  and the field still produced a visible continuation;
- `ANTI_LENS_AVAILABLE_NOT_TRIGGERED` means negative memory exists but did not
  match the current shape;
- anti-lens evidence never grants truth by itself; it explains what was
  suppressed.

### Future Lenses

- Spectral Lens: mode/frequency contribution readout.
- Microscope Lens: one local triad/branch proof.
- Telescope Lens: far weak-corpus discovery.

## Hot Budget

The hot LLMWave path must keep the NANDA-6M rule:

```text
hot core budget:       <= 6 MiB
active focus window:   <= 15,000 patterns/triads
pattern record target: 32 or 64 bytes
JSON/reporting:        cold layer, not hot loop
```

Cold layer:

- text;
- aliases;
- tokenizer;
- dictionaries;
- corpus focus;
- evidence and documents.

Hot layer:

- packed pattern records;
- centroids;
- lanes;
- replay;
- lens readout state.

## v67 MVP

`nanda-llmwave` must expose:

```json
{
  "llmwave_contract": {
    "version": "v67-field-lens-contract",
    "state": "LLMWAVE_LENS_READY",
    "field": {},
    "selected_lens": {},
    "lenses": {},
    "baseline_compare": {},
    "hot_budget": {}
  }
}
```

Minimum lenses:

- Pattern Lens;
- Polarity Lens;
- Cleanup Lens;
- Token Lens.
- Convex Lens;
- Concave Lens;
- Prism Lens.
- Role Lens;
- Temporal Lens;
- Evidence Lens;
- Energy Lens;
- Anti Lens.

Minimum baselines:

- lexical token overlap;
- graph next-edge hint;
- current decode top pattern.

## Success Criteria

v67 is done when:

- `nanda-llmwave --lens pattern` reports a v67 contract;
- the contract exposes field, lens, baseline, hot budget, and proof state;
- Pattern/Polarity/Cleanup/Token/Convex/Concave/Prism lenses have explicit
  states;
- ambiguous or reversed readouts return WATCH, not forced PASS;
- tests verify v67 fields on the existing LLMWave corpus;
- no existing v60 proof behavior regresses.

## v80 Optics Core

v80 is done when:

- `field_snapshot.version == "v77-field-snapshot"`;
- `lens_taxonomy.version == "v76-lens-taxonomy"`;
- `--lens convex` reports `v78-convex-gathering-lens`;
- `--lens concave` reports `v79-concave-separation-lens`;
- `--lens prism` reports `v80-prism-explanation-lens`;
- local tests verify the three optics lenses on the route-trap fixture.

## v85 Semantic Optics

v85 is done when:

- `--lens role` reports `v81-role-binding-lens`;
- `--lens temporal` reports `v82-temporal-order-lens`;
- `--lens evidence` reports `v83-evidence-binding-lens`;
- `--lens energy` reports `v84-energy-stability-lens`;
- `--lens anti` reports `v85-anti-lens-destructive-report`;
- local tests verify all five lenses on route-trap and reject-memory fixtures.

## Research Anchors

- HRR / Plate: binding and unbinding are role/filler lenses.
- VSA / HDC: bundling, binding, permutation, cleanup are field operations.
- Kanerva SDM: address activation is a memory lens.
- Hopfield / modern associative memory: energy readout is a stability lens.
- Superposition work: compression must be measured with crosstalk.
- Fourier / Nanda grokking: wave claims require ablation and baselines.
