# LLMWave-Big Roadmap

Status: planning document.
Updated: 2026-06-20.

Engineering rules for implementing this roadmap are in
`LLMWAVE_BIG_ENGINEERING_RULES.md`. Follow them before coding v158+.

LLMWave-Big is not a small model. It is a large cognitive wave model with a
small cache-resident active core.

```text
Big Wave Atlas        = long-term memory: facts, schemas, documents, code
Hot Cognitive Core    = active 6-8 MB cache-resident focus
L2 Word Field         = fast tokens, roots, morphemes, surface forms
L3 Schema Field       = active operators, roles, routes, semantic schemas
Consolidation Engine  = experience -> schemas + residuals
Loader                = pulls the right cognitive packet into the active core
```

The goal is not to fit everything into L3 cache. The goal is to keep the active
mind in cache while the larger Wave Atlas provides long-term memory.

```text
large long-term memory
  -> focused active packet
  -> schema/residual compression
  -> L2/L3 generation and reasoning
```

## Phase 0: Big Model Contract, v158-v160

### v158 Big Model Contract

Define the main layers:

```text
Wave Atlas        cold/warm long-term memory
Active Core       6-8 MB hot cognitive focus
L2 Word Field     fast language surface
L3 Schema Field   active schemas/operators
Residual Store    private fact traces
Consolidator      compression and sleep
Loader            active focus selection
```

Hard distinction:

```text
model size != active core size
context size != hot memory size
long-term memory size != cache-resident focus
```

### v159 Bigness Metrics

Track:

```text
atlas_facts_total
schemas_total
operators_total
active_core_bytes
active_schemas
active_residuals
bytes_per_useful_fact
useful_inference_per_mb
schema_reuse_ratio
residual_saving_ratio
cognition_score
```

Core score:

```text
cognition_score =
  recall
+ inference
+ role_safety
+ contradiction_veto
+ compression_gain
- false_positive
```

### v160 Claim Boundary

Allowed states:

```text
BIG_MODEL_NOT_PROVEN
BIG_MEMORY_INDEXED
ACTIVE_CORE_WORKS
SCHEMA_COMPRESSION_WORKS
COGNITIVE_LIFT_CANDIDATE
LLMWAVE_BIG_CANDIDATE
```

Implementation checkpoint:

```text
v219-v230: implemented as nanda-llmwave-big eval
state: COGNITIVE_LIFT for the built-in eval surface
scope: corpus domains, inference, role swap, contradiction, multi-hop,
       missing evidence, generation, style, code, business, cognitive score
not claimed: LLM_READY, LLMWAVE_BIG_CANDIDATE, nonlinear memory proof
```

No "we built an LLM" claim before eval.

Implementation checkpoint:

```text
v158-v160: implemented as nanda-llmwave-big contract
state: BIG_MODEL_NOT_PROVEN
scope: contract, metrics, L2/L3 boundary, claim firewall
not claimed: LLM_READY, NONLINEAR_MEMORY_PROVEN, CACHE_ONLY_EXECUTION_PROVEN
```

## Phase 1: Wave Atlas, v161-v170

### v161 Atlas File Format

Long-term memory layout:

```text
atlas/
  symbols.bin
  operators.bin
  schemas.bin
  residuals.bin
  evidence.bin
  cartridges/
    code.bin
    business.bin
    customs.bin
    language_ru.bin
    language_en.bin
```

Records:

```text
SymbolAtom
  id: u32
  kind: u8
  lang: u8
  flags: u16
  wave_seed: u32
  alias_root: u32

OperatorAtom
  id: u16
  arity: u8
  polarity_rules: u16
  phase: u16
  inverse_id: u16

SchemaRecord
  id: u32
  operator_id: u16
  subject_role: u16
  object_role: u16
  route_id: u16
  centroid_id: u32
  confidence: u8

ResidualRecord
  schema_id: u32
  subject_id: u32
  object_id: u32
  phase_delta: i16
  evidence_ref: u32
```

The atlas may be large. The loader must return a small active packet.

### v162 Symbol Dictionary

Symbol kinds:

```text
word
root
morpheme
entity
document
role
source
route
time
number
```

L2 and L3 share symbol IDs but use different projections.

### v163 Operator Dictionary

Cognitive operators:

```text
requires(A,B)
supports(A,B)
issues(A,B)
pays(A,B)
owns(A,B)
causes(A,B)
contradicts(A,B)
same_as(A,B)
part_of(A,B)
before(A,B)
after(A,B)
source_weaker_than(A,B)
role_swap(A,B)
```

Each operator owns:

```text
phase
arity
directionality
inverse
allowed_subject_roles
allowed_object_roles
anti_rules
```

### v164 Schema Atlas

Schemas are cognitive forms, not individual facts:

```text
supplier issues invoice
buyer pays supplier
declaration requires documents
certificate supports clearance
function calls dependency
config controls runtime
```

### v165 Residual Store

Private facts are residuals attached to schemas:

```text
Honglu issued PI-03
Rustrade pays Honglu
Fanta made by Huizhou plant
function A calls function B
```

### v166 Evidence Store

Evidence stays cold:

```text
source_file
line
timestamp
confidence
canonical/current/archive
```

Only `evidence_ref` enters the active core.

### v167 Cartridge Banks

Domain cartridges:

```text
language_ru
language_en
code_rust
business_docs
customs
finance
personal_project
```

### v168 Atlas Index

Indexes:

```text
symbol -> schemas
operator -> schemas
route -> schemas
entity -> residuals
query_wave -> candidate schemas
```

### v169 Atlas Doctor

Detect:

```text
duplicate symbols
overloaded operators
schema too broad
residual too isolated
source conflict
route imbalance
```

### v170 Atlas Loader

Input:

```text
query -> active packet
```

Output:

```text
top symbols
top operators
top schemas
top residuals
negative lanes
evidence refs
```

Implementation checkpoint:

```text
v161-v170: implemented as nanda-llmwave-big atlas
state: ATLAS_CONTRACT_READY_NOT_HOT_RUNTIME
scope: file layout, records, symbol/operator/schema/residual reports,
       cold evidence refs, cartridges, indexes, doctor, loader preview
not claimed: hot runtime, cache-only execution, nonlinear memory proof
```

## Phase 2: Hot Active Core, v171-v180

### v171 Active Core Contract

Hot core:

```text
6-8 MB
1024D wave basis
active schemas
active residuals
active operators
L2 local candidates
anti-lanes
scratch/workspace
```

### v172 Active Packet Format

```text
ActivePacket
  symbols: N
  operators: N
  schemas: N
  residuals: N
  lanes: N
  evidence_refs: N
```

Requirements:

```text
fits L3
fast scan
no strings
no JSON
```

### v173 Active Schema Projection

```text
schema_wave =
  operator_wave
+ subject_role_wave
+ object_role_wave
+ route_wave
+ phase
+ polarity
```

### v174 Active Residual Projection

```text
residual_wave =
  schema_wave
+ subject_entity_delta
+ object_entity_delta
+ evidence_bias
```

### v175 Active Loader Eval

Example:

```text
query: who issued the invoice?
loader must lift:
  issues operator
  supplier role
  invoice schema
  relevant residuals
```

### v176 Focus Competition

If the loader lifts too much, resolve:

```text
schema competition
route competition
evidence competition
```

Use multi-axis top-k, not pure similarity.

### v177 Active Core Budget

Draft budget:

```text
symbols          512 KB
operators        256 KB
schemas        1-2 MB
residuals      1-2 MB
centroids        1 MB
lanes          512 KB
workspace      1 MB
```

### v178 Hot Core Runtime

Operations:

```text
excite(query)
settle()
peak_detect()
anti_veto()
reconstruct()
answer_plan()
```

### v179 Core Benchmark

```bash
nanda bench6m --mode active-core
```

Metrics:

```text
ns/query
cache_misses
schemas/sec
residuals/sec
peak_stability
```

### v180 Core Verdict

```text
ACTIVE_CORE_READY
ACTIVE_CORE_THIN
ACTIVE_CORE_CONTESTED
ACTIVE_CORE_SPILL
```

Implementation checkpoint:

```text
v171-v180: implemented as nanda-llmwave-big active-core
bench: nanda-bench6m --mode active-core
state: ACTIVE_CORE_READY for the built-in typed sample cycle
scope: ActivePacket records, 6 MiB budget, schema/residual projection,
       loader eval sample, focus competition, runtime ops, microbenchmark
not claimed: full LLM, nonlinear memory proof, Atlas-scale cache-only execution
```

## Phase 3: L2 Word Field, v181-v190

### v181 L2 Word Atlas

L2 owns:

```text
tokens
roots
morphemes
forms
synonyms
language variants
prefix continuations
```

### v182 Active L2 Slice

L2 keeps only the active slice:

```text
current language
current domain
current prefix
current style
```

### v183 Prefix Wave

Each character updates:

```text
prefix_wave
local_context_wave
candidate_wave
```

### v184 L2 Candidate Cache

```text
top 128-4096 token candidates
```

### v185 L3 Bias Into L2

L3 provides:

```text
route = business_doc
operator = issues
role expectation = supplier/document
style = formal_ru
```

L2 chooses:

```text
выставил
инвойс
счёт
PI
```

### v186 L2 Anti-Wave

Suppress candidates that match the prefix but break the active schema.

### v187 L2/L3 Sync

```text
L2 per keystroke
L3 per word boundary / punctuation / semantic shift
```

### v188 Multilingual Surface

One L3 meaning can surface through multiple L2 banks:

```text
RU: поставщик выставляет инвойс
EN: supplier issues invoice
CN: 供应商开具发票
```

### v189 L2 Eval

Check:

```text
prefix accuracy
semantic consistency
role safety
language switch
```

### v190 L2 Runtime Verdict

```text
L2_READY
L2_NEEDS_L3
L2_AMBIGUOUS
```

Implementation checkpoint:

```text
v181-v190: implemented as nanda-llmwave-big l2
state: L2_READY for the built-in L3-biased prefix sample
scope: L2 ownership, active slice, prefix wave, candidate cache,
       L3 bias, anti-wave, sync policy, multilingual surface, eval metrics
not claimed: natural language generation, full tokenizer, L3 schema storage in L2
```

## Phase 4: Schema/Residual Nonlinear Write, v191-v205

### v191 Write Decomposition

Example:

```text
Honglu issued PI-03 to Rustrade
```

Decompose:

```text
entity: Honglu
operator: issues
object: PI-03
role: supplier/document
route: business_docs
```

### v192 Reconstructability Score

```text
score =
  schema_match
+ role_match
+ operator_match
+ entity_known
+ evidence_confidence
- false_positive_risk
```

### v193 Write Decision

```text
if reconstructability high:
  centroid update + small residual
else:
  full residual
```

### v194 Residual Format V1

```text
schema_id: u32
subject_id: u32
object_id: u32
phase_delta: i16
evidence_ref: u32
flags: u16
```

### v195 Centroid Update

Update:

```text
schema centroid
operator centroid
entity role centroid
route centroid
```

### v196 Anti-Residual

False fact:

```text
invoice issued supplier
```

Stores anti-residual:

```text
forbid document-as-subject issues supplier
```

### v197 Schema Promotion

Repeated residual cluster becomes a new schema.

### v198 Schema Split

Over-broad schema splits by route/source/role.

### v199 Write Ablation

Compare:

```text
full record
centroid-only
residual-only
schema+residual
```

### v200 Nonlinear Write Curve

Plot:

```text
facts -> bytes/useful_fact
```

### v201 Compression Safety

Compression is valid only if role errors do not grow.

### v202 Exception Handling

Rare important exceptions stay as high-confidence residuals.

### v203 Source-Aware Write

Current/canonical evidence weighs more than archive/noise.

### v204 Write Bench

```bash
nanda bench6m --mode write-density
```

### v205 Write Verdict

```text
LINEAR_WRITE
RESIDUAL_SAVING
SUBLINEAR_WRITE
COGNITIVE_COMPRESSION
```

Implementation checkpoint:

```text
v191-v205: implemented as nanda-llmwave-big write
bench: nanda-bench6m --mode write-density
state: RESIDUAL_SAVING for the built-in schema+residual sample
scope: decomposition, reconstructability, write decision, residual V1,
       centroid update, anti-residual, promotion/split, ablation,
       write curve, compression safety, source-aware write, microbenchmark
not claimed: SUBLINEAR_WRITE, COGNITIVE_COMPRESSION, nonlinear memory proof
```

## Phase 5: Consolidation / Sleep, v206-v218

### v206 Sleep Pass

```text
full facts -> schemas + residuals
```

### v207 Duplicate Merge

Duplicates reinforce centroids instead of creating more records.

### v208 Alias Merge

```text
invoice / инвойс / PI / proforma
```

### v209 Conflict Preservation

Conflicts are preserved:

```text
source A says X
source B says not X
```

### v210 Schema Strength

Schema strength comes from repetition and evidence.

### v211 Forgetting

Weak residuals decay.

### v212 Anti-Memory

Repeated errors become anti-lanes.

### v213 Consolidation Eval

Before/after:

```text
memory_bytes
recall
inference
false_positives
role_safety
```

### v214 Sleep Benchmark

```bash
nanda bench6m --mode consolidate
```

### v215 Cognitive Compression Score

```text
explained_facts / stored_full_records
```

### v216 Atlas Rebuild

Rebuild indexes after sleep.

### v217 Cartridge Repacking

Domain banks compact independently.

### v218 Consolidation Verdict

```text
CONSOLIDATION_SAFE
CONSOLIDATION_LOSSY
CONSOLIDATION_CONFLICTED
```

Implementation checkpoint:

```text
v206-v218: implemented as nanda-llmwave-big consolidate
bench: nanda-bench6m --mode consolidate
state: CONSOLIDATION_SAFE for the built-in sleep sample
scope: sleep pass, duplicate merge, alias merge, conflict preservation,
       schema strength, forgetting, anti-memory, before/after eval,
       cognitive compression score, Atlas rebuild, cartridge repacking
not claimed: broad cognitive compression proof, lossy conflict resolution
```

## Phase 6: Big Cognition Eval, v219-v230

### v219 Big Eval Corpus

Use complex stories:

```text
documents
money
goods
certification
code
configs
sources
routes
```

### v220 Inference Tasks

Example:

```text
invoice issued by supplier
payment made by buyer
declaration requires invoice

Question:
what does the declaration need and who provides it?
```

### v221 Role-Swap Tasks

Check reversed-role errors.

### v222 Contradiction Tasks

Conflicting sources must produce WATCH.

### v223 Multi-Hop Tasks

```text
A requires B
B supports C
C blocked by D
```

### v224 Missing Evidence Tasks

The model must report missing evidence, not hallucinate.

### v225 Generation Tasks

L3 plan -> L2 text.

### v226 Style Tasks

One meaning, different styles.

### v227 Code Tasks

```text
source module -> public export -> runtime caller
```

### v228 Business Tasks

```text
supplier -> invoice -> payment -> customs
```

### v229 Cognitive Score

```text
recall
inference
role_safety
contradiction_veto
compression_gain
generation_consistency
```

### v230 Big Verdict

```text
BIG_INDEXED
ACTIVE_CORE_READY
COGNITIVE_LIFT
LLMWAVE_BIG_CANDIDATE
```

## Phase 7: Runtime Product, v231-v245

### v231 Local Daemon

```text
atlas loader
active core
L2/L3 loop
consolidation scheduler
```

### v232 Skill Integration

Codex skill can query:

```bash
nanda llmwave-big query ...
```

### v233 Editor / Typing Mode

Typing assistant with L2 proposals and L3 veto/rerank.

### v234 Code Review Mode

Structural relations in code.

### v235 Business Graph Mode

Documents, payments, roles, obligations.

### v236 Memory Import

Import:

```text
markdown
json
text
code
```

### v237 Memory Export

Export cartridges and active packets.

### v238 Personal Atlas

Project memory for the user.

### v239 Domain Atlas

Separate domain memory.

### v240 Safety

Do not answer if the field is contested.

### v241 Explainability

Show:

```text
schema
residual
anti-wave
source
```

### v242 Performance

Target:

```text
hot query < 10 ms local
```

### v243 Big Load Test

```text
1M facts atlas
active core still small
```

### v244 Release Candidate

Docs, examples, reproducible eval.

### v245 LLMWave-Big v1

Criteria:

```text
large long-term memory
small active core
schema/residual nonlinear write
cognitive eval passes
```

Implementation checkpoint:

```text
v231-v245: implemented as nanda-llmwave-big query
state: LLMWAVE_BIG_V1_CANDIDATE for the runtime product surface
scope: local daemon contract, skill integration, typing mode, code review,
       business graph, import/export, personal/domain Atlas, safety,
       explainability, performance target, load-test contract, RC checklist,
       v1 criteria
not claimed: LLM_READY, nonlinear memory proof, cache-only execution proof
```

## Hard Problems

### Nonlinear Write Without Hallucination

Risk:

```text
over-compression loses private facts
```

Mitigation:

```text
schema + residual + evidence_ref
```

### L2/L3 Separation

Risk:

```text
L2 suggests a word that breaks the meaning
```

Mitigation:

```text
L3 semantic veto
```

### Big Atlas, Small Core

Risk:

```text
loader misses the needed schema
```

Mitigation:

```text
multi-axis retrieval:
symbol + operator + route + evidence + anti-lanes
```

### Consolidation

Risk:

```text
exception becomes part of a broad schema
```

Mitigation:

```text
exception residuals
source weighting
conflict preservation
```

### Proof Of Big Cognition

Recall is not enough. Measure:

```text
recall
inference
role_safety
contradiction_veto
compression_gain
generation_consistency
```

## Summary

```text
Wave Atlas is large.
Hot Core is small.
L2 speaks.
L3 thinks.
Residuals store private facts.
Schemas store general cognition.
Consolidation grows intelligence.
Anti-wave suppresses false routes.
```

LLMWave-Big is a large model with a cache-resident active mind, not a small
model with a large table.
