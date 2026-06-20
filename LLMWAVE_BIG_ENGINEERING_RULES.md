# LLMWave-Big Engineering Rules

Status: mandatory rules for v158+ work.
Updated: 2026-06-20.

This document exists to keep LLMWave-Big from turning into a larger pile of
CLI/report code. The goal is a large cognitive wave model with a small active
hot core, not a bigger table of packed records.

## Pre-Coding Gate

Before starting a non-trivial LLMWave-Big block:

```bash
nanda dogfood . --refactor-plan --format json
nanda map-code <target-file> --format json
```

Acceptable pre-coding state:

```text
dogfood.agent_decision.safe_to_edit = true
foreign_pull = 0
root WATCH is size-only
local branches PASS
```

If a target file is HIGH-risk in `nanda map-code`, do not add new architecture
there. Extract a boundary first.

Current pre-pass findings:

```text
project dogfood: SAFE_TO_EDIT
src/pattern_store.rs: HIGH risk, too broad for new LLMWave-Big architecture
src/pack6m.rs: MEDIUM risk, keep as cold-to-hot bridge
src/nanda_6m.rs: LOW risk but hot-core only
src/bench6m.rs: LOW risk, benchmark entrypoint
```

## Module Boundaries

New LLMWave-Big work must use explicit module boundaries:

```text
src/llmwave_big/
  mod.rs
  atlas.rs
  symbols.rs
  operators.rs
  schemas.rs
  residuals.rs
  loader.rs
  active_core.rs
  l2_word_field.rs
  l3_schema_field.rs
  write.rs
  consolidation.rs
  eval.rs
  report.rs
```

Allowed responsibilities:

```text
atlas.rs          long-term memory containers and cartridge metadata
symbols.rs        SymbolAtom and symbol projection
operators.rs      OperatorAtom, arity, phase, polarity rules
schemas.rs        SchemaRecord and schema projection
residuals.rs      ResidualRecord, anti-residuals, evidence refs
loader.rs         query -> ActivePacket
active_core.rs    hot active packet excitation, settle, peak, reconstruct
l2_word_field.rs  token/root/morpheme candidates
l3_schema_field.rs operators, routes, role expectations
write.rs          reconstructability and residual-only write
consolidation.rs  sleep pass, schema promotion/split, decay
eval.rs           cognition eval and verdicts
report.rs         JSON/text report assembly
```

Do not add new LLMWave-Big architecture into `src/pattern_store.rs`.
Only use it as a legacy surface until commands are extracted.

## Hot-Core Rules

Hot code means cache-resident runtime code. It must obey:

```text
no JSON
no strings
no heap allocation in the inner loop
no hash maps in the inner loop
fixed-size records
explicit byte budget
deterministic output
bench6m coverage
```

Hot-core data structures go in:

```text
src/nanda_6m.rs
src/nanda_6m/
src/llmwave_big/active_core.rs
```

Cold-to-hot translation goes in:

```text
src/pack6m.rs
src/llmwave_big/loader.rs
```

## Atlas Rules

The Wave Atlas is allowed to be large. It is not required to fit L3.

Atlas records may use files, indexes, dictionaries, and cartridges. The active
packet produced by the loader must be small and hot-core compatible.

Required separation:

```text
cold labels/evidence/text  -> Atlas
compact IDs/phases/seeds   -> ActivePacket
hot wave operations        -> Active Core
```

Do not put evidence text, source snippets, or JSON into active-core records.
Use `evidence_ref`.

## L2/L3 Rules

L2 and L3 must stay separate:

```text
L2 = word/token/root/morpheme surface
L3 = schema/operator/role/route cognition
```

L2 can suggest forms. L3 can veto/rerank forms. L2 must not decide role truth.
L3 must not spell every word.

Expected runtime cadence:

```text
L2: per keystroke / local prefix update
L3: word boundary / punctuation / semantic shift
```

## Nonlinear Write Rules

The core write question is:

```text
can the field reconstruct this fact from existing schemas?
```

Write decision:

```text
if reconstructability high:
  update centroids
  write small residual
else:
  write full residual
```

Required metrics:

```text
bytes_per_useful_fact
schema_reuse_ratio
residual_saving_ratio
role_error_rate
false_positive_rate
inference_score
```

Never claim nonlinear memory from storage size alone.

## Consolidation Rules

Consolidation is allowed to compress repeated facts into schemas, but it must
preserve exceptions and conflicts.

Required behavior:

```text
duplicates reinforce centroids
aliases merge only with evidence
conflicts become WATCH, not deletion
rare exceptions keep high-confidence residuals
repeated errors become anti-lanes
```

## Claim Boundary

Allowed claim states:

```text
BIG_MODEL_NOT_PROVEN
BIG_MEMORY_INDEXED
ACTIVE_CORE_WORKS
SCHEMA_COMPRESSION_WORKS
COGNITIVE_LIFT_CANDIDATE
LLMWAVE_BIG_CANDIDATE
```

Forbidden claims until eval proves them:

```text
LLMWave is an LLM
nonlinear memory is proven
cache-only execution is proven
100k records means cognition
phase coherence means understanding
```

## Required Tests For Each Major Block

For code changes:

```bash
cargo fmt
scripts/test-local.sh
cargo clippy --all-targets --all-features -- -D warnings
scripts/test-edge-cases.sh
nanda dogfood . --refactor-plan --format json
scripts/install-local.sh
nanda-self-check
```

For docs-only changes:

```bash
git diff --check
nanda dogfood . --refactor-plan --format json
```

For hot-core changes:

```bash
nanda bench6m --mode <new-mode> --format json
```

## Performance Rules

Cold reports may be slow if they are explicit research reports.
Hot claims require hot benchmarks.

Do not use `llmwave-memory density` timing as hot-core proof. It is a cold JSON
report. Use `bench6m` modes for hot timing.

## Refactor Stop Conditions

Stop and refactor before adding features when:

```text
target file is HIGH-risk in nanda map-code
new module crosses Atlas/ActiveCore/L2/L3 boundary
new feature needs JSON inside hot loop
new verdict depends on untested metric
new code would make pattern_store.rs larger
```

## Implementation Order

Follow `LLMWAVE_BIG_ROADMAP.md` in order:

```text
v158-v160   contract and metrics
v161-v170   Wave Atlas
v171-v180   Hot Active Core
v181-v190   L2 Word Field
v191-v205   Schema/Residual nonlinear write
v206-v218   Consolidation / sleep
v219-v230   Big cognition eval
v231-v245   Runtime product
```

The first implementation step should create module skeletons and contracts,
not another report layer in `pattern_store.rs`.

Current v158-v170 commands:

```bash
nanda llmwave-big contract --format json
nanda llmwave-big atlas --format json
```

This command must stay contract-only until Wave Atlas, Active Core, L2/L3,
residual write, consolidation, and big cognition eval provide measured verdicts.
