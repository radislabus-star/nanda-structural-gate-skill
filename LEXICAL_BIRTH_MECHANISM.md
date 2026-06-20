# LLMWave Lexical Birth Mechanism

Status: mechanism contract, not proof of corpus learning.

This document fixes the base idea behind word birth and memory in LLMWave. The
goal is to keep the mechanism grounded in language-acquisition, psycholinguistic,
connectionist, and associative-memory literature, while preserving the NANDA
claim boundary.

## Core Idea

A word is not born when a UTF-8 string, hash, or numeric handle appears.

A word is born when a surface form becomes a stable memory binding:

```text
surface form
  + meaning/context center
  + usage traces
  + grammar behavior
  + cleanup/attractor recovery
  + anti-confusion safety
  -> accepted lexical unit
```

The base idea is taken from several lines of work:

- lexical access separates concept, lemma, and word form;[^levelt]
- connectionist word models bind written/spoken form to meaning as distributed
  activation, not as one isolated string;[^plaut]
- infants can segment candidate word forms from continuous speech through
  statistical regularities;[^saffran]
- word-referent uncertainty can be resolved across many ambiguous situations,
  not only in one perfect observation;[^smith-yu]
- grammar and lexicon develop together rather than as fully separate modules;[^bates-goodman]
- usage and frequency strengthen lexical storage through repeated traces;[^bybee]
- self-organizing lexical models bind phonological and semantic maps through
  learned association;[^devlex]
- associative memory gives the engineering criterion: noisy or partial cues must
  settle back to the same stable basin.[^hopfield]

## Storage Mechanism

LLMWave stores a word as layered memory, not as only text.

Cold memory owns the productive and evidence-heavy parts:

```text
surface production atoms: graphemes, bytes, morphemes, roots, endings
surface programs: ordered atom recipes, casing, punctuation, script flags
exact copy spans: observed bytes for names, codes, rare forms
observation traces: where the candidate appeared
context evidence: surrounding words, roles, routes, documents
grammar frames: how the candidate behaves syntactically
```

Hot memory sees only compact fields:

```text
symbol_id
surface_hash
lemma_id
concept_centroid_id
context_centroid_id
cleanup_target_id
root_id
morpheme_id
syntactic_frame_id
evidence_ref count
```

This is why `surface_hash` alone is not enough. A hash can compare or route a
candidate, but it cannot spell the word. Text output needs a surface production
path:

```text
common form -> compose from grapheme/morpheme atoms
rare exact form -> copy observed evidence span
unknown form -> fall back to bytes/chars
```

A numeric handle may exist inside one runtime packet, but it is not the storage
principle and not the source of the word.

## Birth Mechanism

### 1. Segmentation

Input is a stream of characters, bytes, syllables, tokens, or morphemes.

The system first detects a recurring surface chunk:

```text
stream -> boundary statistics -> provisional surface candidate
```

This follows statistical word segmentation: a learner can use transition
statistics to infer word-like units from continuous input.[^saffran]

LLMWave gate:

```text
segmentation_score >= threshold
```

If the surface cannot be segmented reliably, no word candidate is opened.

### 2. Fast Mapping

The new surface receives a weak provisional symbol if it appears with a plausible
referent, role, action, or scene:

```text
surface candidate + first plausible meaning hint -> provisional symbol_id
```

This is not acceptance. It is only a temporary handle, matching the idea that an
early word guess can be incomplete and later revised.

LLMWave gate:

```text
fast_mapping_score >= threshold
```

### 3. Cross-Situational Convergence

The same candidate must appear in multiple situations. If different situations
share one common context center, the candidate gains meaning stability:

```text
context A: word near x/y/z
context B: word near x/k/m
context C: word near x/p/q
shared center: x
```

This follows cross-situational word learning: repeated ambiguous observations can
resolve a word-referent mapping over time.[^smith-yu]

LLMWave gate:

```text
cross_situational_score >= threshold
```

### 4. Usage / Exemplar Strengthening

A word candidate must accumulate traces. Frequency and repeated use are not just
counts; they change how strongly and quickly the word is retrieved.[^bybee]

LLMWave operation:

```text
new observation -> usage_score += weighted trace
```

Evidence does not need to be identical. Similar observations reinforce the same
candidate when they land near the same context centroid.

LLMWave gate:

```text
usage_score >= threshold
```

### 5. Grammar Integration

The candidate must behave consistently in grammar or schema frames:

```text
candidate acts like noun / verb / role / document / operator
candidate occupies stable syntactic or schema slots
```

This follows the grammar-lexicon coupling line: lexical growth and grammar
growth are deeply connected.[^bates-goodman]

LLMWave operation:

```text
candidate -> lemma_id + syntactic_frame_id
```

LLMWave gate:

```text
grammar_score >= threshold
```

### 6. Attractor / Cleanup Recovery

The candidate must be recoverable from noisy or partial cues:

```text
partial cue -> field settles -> same candidate
```

This is the memory criterion. If the word only appears under one exact prompt,
it is not stable memory. It must form a recoverable basin.[^hopfield]

LLMWave operation:

```text
candidate -> cleanup_target_id
```

LLMWave gate:

```text
attractor_margin >= threshold
```

### 7. Anti-Confusion

The candidate must not steal another word's basin. A new word that is always
confused with a stronger existing word remains provisional.

LLMWave operation:

```text
compare nearby basins
measure collision/confusion
apply anti-wave penalty
```

LLMWave gate:

```text
anti_confusion_penalty <= threshold
```

## Acceptance Rule

The word is accepted only when all core gates pass:

```text
segmentation
fast mapping
cross-situational convergence
usage strength
grammar integration
attractor cleanup
anti-confusion
```

Then and only then:

```text
LexicalBirthCandidate32 -> LexicalBindingRecord32
```

If one of the gates fails, the candidate remains provisional or is rejected as
noise.

## Memory Birth

Memory is born in layers:

```text
observation trace
  -> provisional candidate
  -> context centroid
  -> usage-strengthened trace bundle
  -> grammar/schema binding
  -> cleanup target
  -> stable lexical memory
```

This is the practical version of the literature:

- a word has form and meaning layers, not only a string;[^levelt]
- form and meaning can be represented as coupled distributed fields;[^plaut]
- repeated use and frequency strengthen storage;[^bybee]
- lexical and grammatical structure co-develop;[^bates-goodman]
- stable memory must recover from partial/noisy cues.[^hopfield]

## Current Implementation

The current implementation is:

```text
src/llmwave_big/lexical_birth.rs
src/llmwave_big/surface_production.rs
src/llmwave_big/surface_reconstruct.rs
src/llmwave_big/surface_corpus_eval.rs
```

The current inspection commands are:

```bash
nanda-llmwave-big word-birth --format json
nanda-llmwave-big surface-production --format json
nanda-llmwave-big surface-reconstruct --format json
nanda-llmwave-big surface-corpus-eval --format json
```

The implemented records are:

```text
LexicalBirthCandidate32
LexicalBindingRecord32
SurfaceAtom16
SurfaceProgram32
EvidenceCopySpan24
SurfaceProductionCandidate32
```

`LexicalBirthCandidate32` is a provisional candidate. It stores compact scores
and IDs used to decide whether a surface fragment can become a word.

`LexicalBindingRecord32` is the accepted binding. It connects the surface hash,
lemma, concept centroid, context centroid, cleanup target, morphology, grammar
frame, and evidence count. It still does not claim that visible spelling is
stored as a flat lookup. Visible spelling belongs to surface production memory.

`SurfaceAtom16` stores compact form atoms: grapheme, byte fallback, root, stem,
suffix, or ending. `SurfaceProgram32` is the ordered recipe that composes a
visible form from those atoms. `EvidenceCopySpan24` is the exact recovery path
for names, codes, and one-off strings observed in evidence. `SurfaceProductionCandidate32`
scores whether the current field should compose, copy, or fall back to bytes.
The hot core sees compact ids, scores, hashes, and byte-span refs; UTF-8
materialization stays outside the hot loop.

`surface-reconstruct` is the first materializer check. It expands common forms
from atom programs, recovers rare codes from evidence spans, and uses byte
fallback for unknown forms. It reports exact-match and baseline metrics, but its
state remains `TOY_RECONSTRUCTION_PASS_NOT_DENSITY_PROOF` until a real corpus
suite shows useful compression and low false-surface rate.

`surface-corpus-eval` is the first density candidate check. It compares direct
lookup, per-form programs, byte-only fallback, and family-template reuse. The
new records `SurfaceFamily32` and `SurfaceBinding8` model productive families:
shared roots and suffixes can create many virtual forms. This is still a
synthetic suite, so `nonlinear_surface_memory_proven` remains false.

## Claim Boundary

Allowed claim:

```text
LLMWave has a staged lexical birth mechanism grounded in language-acquisition
and lexical-memory literature.
```

Forbidden claims:

```text
the model has already learned words from a real corpus
the model can generate text from hashes alone
the model stores words as a flat numeric-handle -> UTF-8 lookup
the model has proven nonlinear lexical memory density
the word-birth gate proves semantic understanding
```

The next proof step is not another metaphor. It is a corpus experiment:

```text
real corpus
  -> surface observations
  -> surface production atoms/programs/copy spans
  -> LexicalBirthCandidate32 records
  -> birth gates
  -> accepted LexicalBindingRecord32 records
  -> retrieval/generation eval
  -> lexical/random/bag-of-words baselines
```

[^levelt]: Willem J. M. Levelt, Ardi Roelofs, and Antje S. Meyer, "A theory of lexical access in speech production", Behavioral and Brain Sciences, 1999. PubMed: https://pubmed.ncbi.nlm.nih.gov/11301520/ PDF: https://www.mpi.nl/world/materials/publications/levelt/Levelt_Multiple_1999.pdf
[^plaut]: David C. Plaut, James L. McClelland, Mark S. Seidenberg, and Karalyn Patterson, "Understanding normal and impaired word reading: Computational principles in quasi-regular domains", Psychological Review, 1996. PubMed: https://pubmed.ncbi.nlm.nih.gov/8650300/ PDF: https://www.cnbc.cmu.edu/~plaut/papers/pdf/PlautETAL96PsyRev.wordReading.pdf
[^saffran]: Jenny R. Saffran, Richard N. Aslin, and Elissa L. Newport, "Statistical learning by 8-month-old infants", Science, 1996. PubMed: https://pubmed.ncbi.nlm.nih.gov/8943209/ PMC discussion: https://pmc.ncbi.nlm.nih.gov/articles/PMC3883431/
[^smith-yu]: Linda B. Smith and Chen Yu, "Infants rapidly learn word-referent mappings via cross-situational statistics", Cognition, 2008. PMC: https://pmc.ncbi.nlm.nih.gov/articles/PMC2271000/
[^bates-goodman]: Elizabeth Bates and Judith C. Goodman, "On the inseparability of grammar and the lexicon: Evidence from acquisition, aphasia and real-time processing", Language and Cognitive Processes, 1997. PDF: https://crl.ucsd.edu/bates/papers/pdf/bates-goodman-1997.pdf
[^bybee]: Joan Bybee, "The Emergent Lexicon", Chicago Linguistic Society, 1998. PDF: https://www.unm.edu/~jbybee/downloads/Bybee1998EmergentLexicon.pdf
[^devlex]: Ping Li, Igor Farkas, and Brian MacWhinney, "Early lexical development in a self-organizing neural network", Neural Networks, 2004. PubMed: https://pubmed.ncbi.nlm.nih.gov/15555870/ Abstract: https://researchportal.hkust.edu.hk/en/publications/early-lexical-development-in-a-self-organizing-neural-network/
[^hopfield]: Hubert Ramsauer et al., "Hopfield Networks is All You Need", ICLR, 2021. arXiv: https://arxiv.org/abs/2008.02217
