# NANDA / LLMWave Research Direction

Status: working research compass after `sparse-triad-v6.0-llmwave-proof`.
Updated: 2026-06-19.

This file records the outside research line that should guide the next NANDA
steps. It is not a novelty claim. It is the project memory for what we should
learn, import, test, or deliberately avoid.

## Position

NANDA is not just "holographic memory".

Classical holographic and vector-symbolic memory shows that symbols and
role-filler structures can be stored in high-dimensional superposition and
retrieved by a key. NANDA uses that family of ideas, but the product direction
is narrower and more operational:

```text
facts / code / documents
  -> triad field
  -> interference peaks
  -> destructive and constructive lanes
  -> proof / gate / replay decision
  -> compact packed runtime
```

The core question is not "can we store an answer?" The core question is:

```text
does the relation shape form a stable, explainable peak after competing routes,
negative shortcuts, aliases, role direction, and corpus noise are accounted for?
```

That makes NANDA closer to a wave structural verifier and LLMWave layer than to
a plain associative store.

## Literature Anchors

The word-birth mechanism is fixed in
[`LEXICAL_BIRTH_MECHANISM.md`](LEXICAL_BIRTH_MECHANISM.md), with explicit
footnotes to the language-acquisition and lexical-memory sources that motivate
the staged gates.

The surface-production mechanism is implemented in
`src/llmwave_big/surface_production.rs` and exposed through:

```bash
nanda-llmwave-big surface-production --format json
```

This locks the form-memory direction: the model must produce visible forms from
grapheme/byte atoms, morpheme atoms, surface programs, and exact evidence-copy
spans. It must not fall back to a flat numeric-handle-to-UTF-8 lookup as the
core memory story.

The first reconstruction check is:

```bash
nanda-llmwave-big surface-reconstruct --format json
```

It measures whether common forms, rare evidence-copy forms, and byte fallback
can be materialized correctly. Treat the current result as a toy materializer
pass only; the density claim needs corpus-scale exact-match, false-surface,
copy-error, reuse, and bytes-per-useful-surface evidence.

The first corpus-scale density candidate check is:

```bash
nanda-llmwave-big surface-corpus-eval --format json
```

This introduces family-template reuse: `SurfaceFamily32` plus `SurfaceBinding8`
lets shared roots and suffixes produce many virtual forms. The scientific claim
is still narrow: synthetic family reuse is visible, but nonlinear surface memory
is not proven until a real corpus beats direct lookup and per-form program
baselines under exact-match and false-surface constraints.

The first observed-form bank builder is:

```bash
nanda-llmwave-big surface-bank-build --format json
nanda-llmwave-big surface-bank-validate --format json
nanda-llmwave-big surface-bank-fixture --corpus examples/llmwave-big-surface-corpus.json --format json
nanda-llmwave-big surface-bank-fixture --corpus examples/llmwave-big-surface-corpus-ru.json --format json
nanda-llmwave-big surface-raw-induce --corpus examples/llmwave-big-raw-surface-corpus-ru.json --format json
nanda-llmwave-big surface-raw-induce --corpus examples/llmwave-big-raw-surface-corpus-ru-noisy.json --format json
```

It promotes suffix families from a small embedded corpus and rejects fragments
that require copy or provisional handling. The validator then applies positive
held-out controls, false-family traps, rare-code traps, and order-shuffle
stability checks. The fixture command moves those checks into an external JSON
corpus file. The Russian fixture adds Cyrillic business forms and exact
regulatory identifiers, which is the first multilingual pressure test for this
surface-memory path.
`surface-raw-induce` removes explicit root fields from the input and asks the
engine to induce the family roots from raw Cyrillic forms plus suffix inventory.
The noisy variant adds near-root collisions and must keep those roots rejected
until they have enough independent form evidence.

### Holographic Reduced Representations

Sources:

- Tony A. Plate, "Holographic Reduced Representations", IEEE Transactions on
  Neural Networks, 1995.
  https://redwood.berkeley.edu/wp-content/uploads/2020/08/Plate-HRR-IEEE-TransNN.pdf
- Tony A. Plate, "Holographic Reduced Representations: Convolution algebra for
  compositional distributed representations", IJCAI, 1991.
  https://www.ijcai.org/Proceedings/91-1/Papers/006.pdf
- Ashwinkumar Ganesan et al., "Learning with Holographic Reduced
  Representations", NeurIPS, 2021.
  https://proceedings.neurips.cc/paper_files/paper/2021/file/d71dd235287466052f1630f31bde7932-Paper.pdf

What to import:

- Binding and unbinding should become a first-class experiment, not only an
  analogy. For NANDA that means role/value, route/value, and evidence/value
  lanes that can be composed and then queried back.
- Circular convolution / correlation is the natural candidate for testing
  compact role-filler memory. If used, benchmark it against the current sparse
  triad field, not instead of it.
- Cleanup memory is directly relevant to `pattern_bank`: retrieval should not
  stop at the raw nearest peak. It should pass through a known-clean structural
  dictionary and report uncertainty when cleanup is ambiguous.
- The 2021 HRR learning paper is a warning: naive HRR learning can be
  numerically unstable. If we add trainable HRR-like operations, we need a
  projection or normalization step and explicit stability tests.

What not to claim:

- HRR can retrieve noisy bound structures, but that is not proof that a decoded
  route is true.
- A high dot product or phase match is a retrieval signal, not final evidence.

### VSA / Hyperdimensional Computing

Sources:

- Denis Kleyko et al., "A Survey on Hyperdimensional Computing aka Vector
  Symbolic Architectures, Part I", ACM Computing Surveys, 2022.
  https://redwood.berkeley.edu/wp-content/uploads/2022/11/2022_CSUR_survey_HDCVSA_part_1.pdf
- Denis Kleyko et al., "A Survey on Hyperdimensional Computing aka Vector
  Symbolic Architectures, Part II", ACM Computing Surveys, 2023.
  https://research.ibm.com/publications/a-survey-on-hyperdimensional-computing-aka-vector-symbolic-architectures-part-ii-applications-cognitive-models-and-challenges

What to import:

- Treat NANDA packets as high-dimensional compositional objects with explicit
  operations: bundling, binding, permutation, cleanup, similarity.
- Measure capacity as an empirical curve: dimensions, record count, lane count,
  route count, noise, and retrieval error.
- Keep symbolic structure visible. NANDA's advantage is that a peak can be
  mapped back to triads, routes, supports, anti-supports, and repair tasks.

What not to import blindly:

- VSA demos often show toy symbol manipulation. NANDA must keep the hard agent
  contract: PASS/WATCH/VETO or ANSWER_READY/WATCH/VETO with evidence-facing
  diagnostics.

### Sparse Distributed Memory

Sources:

- Pentti Kanerva, "Sparse Distributed Memory", MIT Press, 1988.
  https://mitpress.mit.edu/9780262111324/sparse-distributed-memory/
- Pentti Kanerva, "Sparse Distributed Memory and Related Models", 1993.
  https://redwood.berkeley.edu/wp-content/uploads/2020/08/KanervaP_SDMrelated_models1993.pdf
- Bricken and Pehlevan, "Attention Approximates Sparse Distributed Memory",
  NeurIPS, 2021.
  https://arxiv.org/abs/2111.05498

What to import:

- The 15k focused proof window should be treated like an address-activated
  local memory, not a failure to use the full 65k arena.
- Dataset doctor, focus, and route balancing are not UX extras. They are the
  cold-layer equivalent of choosing the right active hard locations before
  reading memory.
- Add capacity tests that measure the critical transition from focused recall
  to noisy recall as route count and duplicate hubs grow.

What not to claim:

- "More triads" is not automatically better. After a saturation point, a larger
  corpus can make peaks less question-specific.

### Hopfield / Modern Associative Memory

Sources:

- Ramsauer et al., "Hopfield Networks is All You Need", ICLR, 2021.
  https://arxiv.org/abs/2008.02217
- Widrich et al., "Modern Hopfield Networks and Attention for Immune Repertoire
  Classification", NeurIPS, 2020.
  https://arxiv.org/abs/2007.13505

What to import:

- Reframe peaks as attractor candidates, then measure whether the field settles
  into a stable basin or only flashes a local lexical peak.
- Add an explicit energy / stability view to recurrent decode and beam decode:
  each step should report whether energy improves, saturates, oscillates, or
  jumps route.
- Beam trajectories are useful because several attractors can be plausible
  before cleanup. Do not collapse too early.

What not to claim:

- A stable attractor is not automatically semantic understanding. It is a
  stable retrieval / relation pattern that still needs evidence and gate state.

### Superposition And Polysemanticity

Sources:

- Elhage et al., "Toy Models of Superposition", 2022.
  https://transformer-circuits.pub/2022/toy_model/index.html
- Anthropic, "Toy Models of Superposition", 2022.
  https://www.anthropic.com/research/toy-models-of-superposition

What to import:

- Superposition is not a marketing word. It is a compression mechanism with an
  interference cost.
- NANDA's destructive lanes are most interesting when they suppress a specific
  false reading shape without killing the whole topic.
- Capacity work must track both useful compression and harmful crosstalk.

What to test:

- Can a compact packed field carry more structural patterns than direct table
  slots while keeping false shortcut rate low?
- Does adding negative shortcut lanes reduce crosstalk without erasing useful
  nearby routes?

### Fourier / Grokking / Nanda

Sources:

- Neel Nanda et al., "Progress measures for grokking via mechanistic
  interpretability", ICLR, 2023.
  https://arxiv.org/abs/2301.05217
- OpenReview page for the same paper.
  https://openreview.net/forum?id=9XFSbDPmdW

What to import:

- Treat "WAW" as a reverse-engineering target, not a vibe. If a wave mechanism
  is real, we should be able to ablate it, measure it, and show that a lexical
  or table baseline fails where the wave field wins.
- Keep progress measures: memorization, circuit formation, cleanup. For NANDA,
  the analogues are raw recall, route formation, and shortcut cleanup.
- Fourier features matter as a proof style: show that the field uses structure
  that can be inspected, not only that the final answer happened to be right.

What not to copy:

- Modular addition is a clean group structure. Business documents, code graphs,
  and mixed evidence are messier. NANDA should borrow the inspection discipline,
  not pretend every problem is modular arithmetic.

### Word Birth / Mental Lexicon

Canonical mechanism note:

- [`LEXICAL_BIRTH_MECHANISM.md`](LEXICAL_BIRTH_MECHANISM.md)

Sources:

- Levelt et al., lexical access model: lexical concept -> lemma -> word form.
- Plaut, McClelland, Seidenberg, Patterson, triangle model of word reading:
  orthography, phonology, and semantics are coupled distributed fields.
- Saffran, Aslin, Newport, statistical learning and word segmentation, 1996.
- Smith and Yu, cross-situational word learning, 2008.
- Bates and Goodman, grammar and lexicon are not cleanly separable, 1997.
- Bybee / usage-based and exemplar lexicon: repeated traces strengthen lexical
  storage.
- DevLex / Li, Farkas, MacWhinney: self-organizing lexical maps bind surface
  and meaning over development.

What to import:

- A word is not born as a UTF-8 string, hash, or numeric handle. It is born as a
  stable binding across surface production, meaning context, syntactic behavior,
  usage evidence, and recoverability.
- Word birth needs staged gates: segmentation, fast mapping, cross-situational
  convergence, usage/exemplar strengthening, grammar integration,
  attractor/cleanup stability, and anti-confusion against nearby words.
- Store provisional words separately from accepted lexical bindings. A noisy
  surface fragment should remain provisional until it survives context and
  anti-confusion tests.
- Keep a surface production path for actual text output: grapheme/byte atoms,
  morpheme atoms, surface programs, and exact evidence-copy spans. A wave seed
  or surface hash can compare a word candidate, but cannot spell it by itself.

What not to claim:

- A high context score is not a word.
- A numeric handle without a surface production path cannot generate text.
- A single observation is not lexical learning.
- A lexical birth mechanism is not proof that a real corpus has learned new
  vocabulary.

## NANDA Design Consequences

The next LLMWave line should preserve these laws:

1. Retrieval and verification are separate. Search may find a peak; proof must
   decide if the peak is answer-ready.
2. The hot 6 MiB runtime is a focused proof engine, not a world model.
3. The cold layer owns text, dictionaries, aliases, evidence, and corpus focus.
4. The hot layer owns packed records, centroids, lanes, replay, and top-k state.
5. Destructive interference must be shortcut-specific, not topic-killing.
6. Constructive interference must reinforce accepted support shape, not just
   raise a route name.
7. Cleanup memory must report ambiguity.
8. Superposition claims require capacity curves.
9. WAW claims require lexical/table/graph baselines.
10. Coherence is evidence of field structure, not proof of truth.

## Study-To-Code Roadmap

Status: first proof baseline implemented in `sparse-triad-v6.0-llmwave-proof`.
The v47-v60 reports are now exposed by `nanda-llmwave`, `nanda-decode`,
`nanda-pattern-bank`, `nanda-llmwave-eval`, and the v61 `nanda-demo` weak-spot
surface. The next research job is to move more of these proof signals into the
NANDA-6M hot loop.

### v47: HRR Binding Sandbox

Implemented baseline: `nanda-llmwave` now reports `hrr_binding` with
role/filler bind-unbind recovery. The deeper version should move this into a
packed hot-cycle fixture:

```text
role vector * filler vector -> bound lane
bound lane * inverse(role) -> recovered filler candidate
```

Success criterion:

- role-swap and route-splice fixtures must still VETO;
- recovered filler should map to the same canonical triad entity when the
  source packet is clean;
- ambiguity should return WATCH, not forced PASS.

### v48: Cleanup Memory For Pattern Bank

Implemented baseline: `nanda-pattern-bank` now reports a `cleanup_memory`
contract, and `nanda-llmwave` maps raw decoded patterns to nearest known
triads. The deeper version should turn this into an explicit cleanup
dictionary:

```text
raw decoded pattern -> nearest accepted pattern(s) -> cleanup verdict
```

Success criterion:

- accepted continuations get reinforced;
- rejected shortcuts stay suppressed;
- contested cleanup emits multiple candidates and a WATCH state.

### v49: Attractor / Energy Trace

Implemented baseline: `nanda-decode` and `nanda-llmwave` expose
`v49-attractor-energy-trace`. The deeper version should extend it to beam
decode and packed runtime state:

```text
step energy, route basin, margin, support, anti-support, route jump
```

Success criterion:

- stable beam routes show improving or saturating energy;
- route traps show jumps, oscillation, or destructive suppression;
- eval suite checks the trace, not only the top string.

### v50: Superposition Capacity Curve

Implemented baseline: `nanda-llmwave` reports active pattern load and estimated
crosstalk. The deeper version should become a full benchmark:

```text
patterns x dimensions x routes x negative lanes -> false shortcut rate
```

Success criterion:

- chart or JSON report showing the useful compression region;
- explicit saturation point where WATCH/FOCUS_REQUIRED becomes correct;
- comparison against direct lookup and lexical baseline.

### v51: Shortcut-Specific Anti-Wave Audit

Implemented baseline: `nanda-llmwave` reports `anti_wave_audit` over matched
continuation suppressions. The deeper version must prove locality under
nearby-valid-route fixtures:

```text
reject false reading shape -> suppress that shape -> preserve nearby valid route
```

Success criterion:

- route-level, group-level, and support-level probes all show the same
  shortcut-local behavior;
- a useful route sharing the topic survives the anti-wave.

### v52: LLMWave Read/Write/Retrieve Demo

Implemented baseline: `nanda-llmwave` now produces a single v60 packet:

```text
write facts -> encode field -> query partial structure -> decode candidates ->
proof gate -> feedback -> replay
```

Success criterion:

- demo beats lexical/table baseline on at least one route-trap case;
- report exposes support, anti-support, cleanup, and replay decision;
- no claim of "understanding" unless the proof gate is answer-ready.

## Practical Reading Rule

When adding new ideas from papers, write a tiny "mechanism note" before code:

```text
paper:
mechanism:
what NANDA imports:
what NANDA refuses:
test fixture:
baseline:
failure state:
```

If a mechanism cannot be tested against a baseline, it stays research notes and
does not become a core claim.
