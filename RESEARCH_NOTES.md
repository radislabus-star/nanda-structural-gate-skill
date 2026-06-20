# NANDA / LLMWave Research Notes

This file keeps the research map honest. It is not a claim that NANDA already
proves nonlinear memory density.

## Relevant Lines Of Work

- Sparse Distributed Memory, Pentti Kanerva.
  High-dimensional distributed addressing, noisy recall, and the practical
  capacity/error tradeoff.
- Dense Associative Memory, Krotov and Hopfield.
  Nonlinear energy functions, attractor dynamics, and capacity beyond classical
  Hopfield limits.
- Vector Symbolic Architectures and HRR.
  Binding/unbinding of structured relations in high-dimensional vectors.
- Oscillatory neural networks.
  Phase locking, phase-coded state, and dynamic ensembles as inspiration for
  relation phase and polarity.

## Implemented In This Repository

- 1024-dimensional deterministic field.
- Packed 6 MB hot-core contract.
- 32-byte packed triad records.
- Relation phase channels.
- Subject/object polarity.
- Bidirectional recall.
- Field decomposition and phase mismatch.
- Density reality check with lexical baseline.

## Not Yet Proven

- Nonlinear memory density.
- Exponential useful capacity.
- Cache-only execution.
- 40-cycle retrieval.
- True oscillatory/Kuramoto dynamics.
- L2/L3 two-contour runtime.

## Current Claim Boundary

The current positive signal is narrow:

> Relation phase plus subject/object polarity can beat a lexical bag-of-words
> baseline on reversed-direction traps.

The next required evidence is stronger:

- field beats lexical, relation-only, and naive-vector baselines;
- margin erosion is slower than baselines under density growth;
- suppression or nonlinear scoring extends useful recall;
- packed hot-loop results are measured without JSON/string overhead;
- perf counters show cache behavior.

## Current Density Verdict Path

`nanda llmwave-memory density` now reports v138-v147:

- read the density report;
- stress reversed/baseline traps;
- track margin erosion;
- verify fixed basis;
- find useful capacity threshold;
- mark anti-wave lift candidates;
- compare packed proxy behavior;
- separate L2 prefix work from L3 semantic bias;
- emit a final nonlinear-density verdict.

`NOT_PROVEN` is an acceptable verdict. It means the guardrail worked and the
system refused to turn a pretty field peak into a stronger claim than the data
supports.
