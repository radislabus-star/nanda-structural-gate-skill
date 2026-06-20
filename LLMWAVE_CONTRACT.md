# LLMWave Field + Lens Contract

Status: v109 implementation contract.
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
- a wave-memory object with write/retrieve/feedback/consolidate/decay/generate
  operations;
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

## Memory Core

`nanda-llmwave-memory` is the first explicit LLMWave memory surface.

```text
triads/text
  -> write
  -> wave_memory
  -> retrieve(prefix)
  -> feedback(accept/reject/watch)
  -> consolidate
  -> decay
  -> generate recurrently
```

The memory object contains:

- `patterns`: structural triad continuations;
- `token_patterns`: prefix -> next token/phrase records;
- `phrase_patterns`: prefix -> phrase records;
- `positive_lanes`: accepted continuation traces;
- `negative_lanes`: rejected shortcut traces;
- `resonance_traces`: WATCH observations;
- `consolidation`: duplicate merge state;
- `decay`: forgetting state;
- `packed_runtime`: v93 6M budget estimate.

Memory versions:

- v86: wave-memory schema;
- v87: write path from packet/text into memory records;
- v88: retrieve path through token/phrase resonance;
- v89: feedback learning;
- v90: consolidation;
- v91: decay/forgetting;
- v92: phrase memory;
- v93: packed 6M memory budget;
- v94: recurrent generation;
- v95: memory eval corpus;
- v96: vocabulary/token space;
- v97: sampler;
- v98: beam generator;
- v99: semantic decoder;
- v100: chat loop;
- v101: training from text;
- v102: memory growth;
- v103: self-correction;
- v104: generator eval;
- v105: real memory file format;
- v106: tokenizer contract;
- v107: model config;
- v108: binary packed memory prototype;
- v109: generator quality eval.

The current memory core is still a cold JSON implementation with explicit 6M
budget reporting. It is not yet the final hot packed runtime, but it gives
LLMWave a concrete memory object that can grow, be corrected, be compacted, and
be tested.

## Generator Surface

The first generator surface is:

```text
memory
  -> vocabulary
  -> retrieve(prefix)
  -> sampler
  -> beams
  -> semantic decoder
  -> chat text
```

Commands:

- `nanda-llmwave-memory vocabulary`;
- `nanda-llmwave-memory inspect`;
- `nanda-llmwave-memory pack memory.json --out memory.llmw.bin`;
- `nanda-llmwave-memory unpack memory.llmw.bin`;
- `nanda-llmwave-memory generate --beam-width N --temperature T`;
- `nanda-llmwave-memory chat --prompt ...`;
- `nanda-llmwave-memory answer --prompt ... --facts N`;
- `nanda-llmwave-memory train corpus.txt`;
- `nanda-llmwave-memory grow memory.json packet.json`;
- `nanda-llmwave-memory correct --reject-token ... --accept-token ...`;
- `nanda-llmwave-memory demo --corpus corpus.txt --prompt ...`;
- `nanda-llmwave-memory eval --suite examples/llmwave-memory-corpus.json`.

This is a tiny LLMWave generator, not a full large language model. It can
continue from its own wave memory, rank beams, apply correction feedback, grow
from new packets/text, and decode a selected path into text.

v110-v114 define the first chat-safe path:

- `v110-prompt-adapter` maps a natural question to an internal memory prefix;
- `v111-semantic-guard` vetoes rejected, repeated, low-margin, or route-shifted
  beams before emission;
- `v112-multi-step-coherence` reports why generation stopped and whether the
  route stayed consistent;
- `nanda-serve` supports cached `llmwave_chat` requests;
- `v114-public-demo-script` trains a tiny corpus, runs chat, applies feedback
  lanes, and validates packed memory.

v115-v119 add Answer Core. `retrieve` is next-token resonance, `generate` is
recurrent continuation, `chat` is prompt-to-continuation, and `answer` is a
grounded response assembled from memory records. Answer output must expose:

- `state`: `ANSWER_READY`, `ANSWER_EMPTY`, or `ANSWER_CONTESTED`;
- `safe_to_answer`;
- selected prompt prefix;
- grounding facts;
- safe and suppressed beams;
- review reasons.

The QA eval corpus checks answer behavior directly, not only continuation text.

v120-v126 move precision into the field core:

- relation phase channels: `requires`, `supports`, `issues`, `pays`;
- subject/object polarity: `subject -> relation -> object` is not reversible;
- bidirectional recall: object-to-subject and subject-to-object questions use
  different target energy;
- field decomposition: subject, relation, object, phrase, polarity,
  bidirectional, and anti energy are reported per fact;
- phase collision detection: `FIELD_SINGLE_PEAK`, `FIELD_MULTI_PEAK`,
  `FIELD_PHASE_MISMATCH`, or `FIELD_EMPTY`;
- core eval includes a reversed-polarity trap such as
  `what does invoice issue?`.

v127-v128 add Density Reality Check. It synthesizes larger LLMWave memories and
measures useful recall, reversed-trap safety, field-vs-lexical baseline,
field-state drift, packed bytes, hot-focus boundary, and timing.

v129-v137 turn that check into a core research probe:

- phase-lock metric over single peaks, phase mismatches, and aligned relation
  evidence;
- noise-pressure counters for suppressed facts, anti-energy, and margin;
- nonlinear scoring candidate reported but not used by the answer core;
- expanded baselines: lexical, relation-only, and naive token-vector;
- typed packed hot-loop proxy without JSON/string scoring;
- perf-counter plan for cycles, instructions, and cache misses;
- focus-window experiment for 15k proof windows versus wider storage;
- L2 local contour spec/prototype for prefix candidates and short-context
  rerank under L3 phase bias.

v138-v147 add the verdict layer over those measurements:

- density report reader for humans and agents;
- baseline stress pack for reversed, lexical, relation-only, and naive-vector
  traps;
- margin-erosion curve across record growth;
- fixed-basis test that keeps `WAVE_DIM` constant and records explicit;
- useful-capacity threshold for the largest stable row seen;
- anti-wave capacity-lift candidate for rows where baselines false-positive
  but the field does not;
- packed-runtime density proxy;
- L2 prefix contour and L3-to-L2 rerank contract;
- nonlinear-density verdict: `NOT_PROVEN`, `WEAK_NONLINEAR_SIGNAL`, or
  `CAPACITY_LIFT_CANDIDATE`.

v148-v157 add the useful-capacity layer:

- adversarial density corpus metadata;
- baseline duel report across field, lexical, relation-only, naive-vector, and
  Markov-like baselines;
- margin-vs-baseline compare;
- anti-wave ablation proxy;
- fixed-basis capacity sweep plan;
- useful-capacity score;
- packed density hot-loop report;
- `nanda bench6m --mode density` for typed hot-loop timing;
- L2 candidate cache;
- L3 phase-bias into L2 rerank contract.

It explicitly does not prove nonlinear density or cache-only execution; those
remain research claims until supported by capacity and perf-counter evidence.
The first useful signal is narrower: relation phase and subject/object polarity
can beat simple baselines on reversed-direction traps.

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

## v95 Memory Core

v95 is done when:

- `nanda-llmwave-memory write` emits `v86-wave-memory-schema`;
- `retrieve` emits `v88-memory-retrieve-path`;
- `feedback` emits `v89-feedback-learning`;
- `consolidate` emits `v90-consolidation`;
- `decay` emits `v91-decay-forgetting`;
- memory records include phrase continuations and `v93-packed-6m-memory`;
- `generate` emits `v94-recurrent-generation`;
- `eval --suite examples/llmwave-memory-corpus.json` emits
  `v95-memory-eval` and passes.

## v104 Generator Core

v104 is done when:

- memory objects emit `v96-vocabulary-token-space`;
- `generate` reports `v97-sampler`, `v98-beam-generator`, and
  `v99-semantic-decoder`;
- `chat` emits `v100-chat-loop`;
- `train` emits `v101-training-from-text`;
- `grow` emits `v102-memory-growth`;
- `correct` emits `v103-self-correction`;
- memory eval emits `v104-generator-eval` and passes.

## v109 Model Core

v109 is done when:

- `inspect` emits `v105-real-memory-file-format` with a schema hash;
- `inspect.tokenizer_contract.version == "v106-tokenizer-contract"`;
- `inspect.model_config.version == "v107-model-config"`;
- `pack` writes a binary `.llmw.bin` prototype with `LLMWAVE1` header;
- `unpack` validates the binary prototype as `PACKED_MEMORY_OK`;
- memory eval emits `v109-generator-quality-eval` and covers direct retrieve,
  feedback shift, text training, memory growth, and decay.

## Research Anchors

- HRR / Plate: binding and unbinding are role/filler lenses.
- VSA / HDC: bundling, binding, permutation, cleanup are field operations.
- Kanerva SDM: address activation is a memory lens.
- Hopfield / modern associative memory: energy readout is a stability lens.
- Superposition work: compression must be measured with crosstalk.
- Fourier / Nanda grokking: wave claims require ablation and baselines.
