# LLMWave Persistent Wave Memory Mechanism

Status: architecture guardrail before implementation.

This note exists to prevent the next chat/memory step from becoming a session
log with wave words on top. In LLMWave, a turn must mutate the wave memory state
through compact learned lanes, schema/residual deltas, and anti-wave records.
Raw dialogue history may exist for audit, but it is not model memory.

## Literature Anchors

### HRR / VSA

Sources recorded in `RESEARCH_DIRECTION.md`:

- Tony Plate, "Holographic Reduced Representations", 1995.
- Tony Plate, "Holographic Reduced Representations: Convolution algebra for
  compositional distributed representations", 1991.
- Ganesan et al., "Learning with Holographic Reduced Representations", 2021.
- Kleyko et al., HDC/VSA survey parts I/II.

Mechanism to import:

```text
role/filler binding
cleanup memory
superposition with explicit crosstalk cost
```

LLMWave consequence:

```text
turn -> intent/role/route/filler wave -> bound memory delta
```

Not allowed:

```text
turn -> append raw text -> call that wave memory
```

### Sparse Distributed Memory

Sources recorded in `RESEARCH_DIRECTION.md`:

- Pentti Kanerva, "Sparse Distributed Memory", 1988.
- Kanerva, "Sparse Distributed Memory and Related Models", 1993.
- Bricken and Pehlevan, "Attention Approximates Sparse Distributed Memory",
  2021.

Mechanism to import:

```text
large cold address space
focused active hard locations
noisy recall threshold
```

LLMWave consequence:

```text
Atlas can be large, but active wave memory must be a focused mutable packet.
```

Not allowed:

```text
more stored turns == better memory
```

### Modern Hopfield / Dense Associative Memory

Sources recorded in `RESEARCH_DIRECTION.md`:

- Ramsauer et al., "Hopfield Networks is All You Need", 2021.
- Widrich et al., modern Hopfield applications, 2020.

Mechanism to import:

```text
attractor candidates
energy/stability trace
settling versus route jump
```

LLMWave consequence:

```text
memory write is valid only if the next field pass changes a measurable basin:
accepted route lifted, false shortcut suppressed, unrelated route preserved.
```

Not allowed:

```text
stable peak == understanding
```

### Superposition

Sources recorded in `RESEARCH_DIRECTION.md`:

- Elhage et al., "Toy Models of Superposition", 2022.

Mechanism to import:

```text
compression with interference cost
polysemantic overlap
shortcut-specific suppression
```

LLMWave consequence:

```text
negative lanes must suppress a false reading shape without killing the whole
topic or nearby valid route.
```

Not allowed:

```text
superposition as a capacity claim without false-positive/crosstalk eval
```

### Fourier / Grokking / Nanda

Sources recorded in `RESEARCH_DIRECTION.md`:

- Neel Nanda et al., "Progress measures for grokking via mechanistic
  interpretability", 2023.

Mechanism to import:

```text
inspectable circuit/circuit-progress discipline
ablation before claim
structure over final answer coincidence
```

LLMWave consequence:

```text
wave-memory write must expose before/after field scores and ablation against
no-memory baseline.
```

Not allowed:

```text
WAW claim without ablation and cheap-baseline comparison
```

## Existing Repo Mechanisms To Reuse

The next implementation must reuse these existing routes instead of inventing a
parallel memory:

```text
src/llmwave_big/write.rs
  Schema/residual write, reconstructability, centroid update, anti-residual.

src/llmwave_big/memory_physics.rs
  AntiWaveMemoryRecord32, collision/noise checks, shortcut-specific suppression.

src/llmwave_big/core_v1_feedback_learning.rs
  CoreV1FeedbackMemoryRecord32, next-pass field change report.

src/llmwave_big/field_runtime.rs
  AppliedMemoryRecord32, PersistedFieldMemory, feedback-aware next pass,
  memory-store, learning-eval, consolidation.

src/llmwave_big/training.rs
  hot ask/learn/chat path where learned records affect the next hot ask.

src/llmwave_big/linux_profile/feedback.rs
  Linux-profile positive/negative feedback lanes over reasoning output.
```

## Architecture Rule

Persistent wave memory is not a transcript.

```text
bad:
  turn text -> session journal -> read previous lines as context

good:
  turn -> query wave -> field activation -> verifier decision
       -> WaveMemoryDelta -> durable wave memory
       -> next field pass uses changed memory
```

An audit log may be written, but it is not the model memory and must not be the
source of improved answers.

## Required Memory Record

The first implementation should introduce a fixed-size chat/field memory delta
that can later become binary hot memory.

Candidate record:

```rust
#[repr(C)]
struct PersistentWaveDelta32 {
    source_id: u32,
    relation_id: u16,
    target_id: u32,
    route_id: u16,
    intent_id: u16,
    phase_delta: i16,
    reinforce_score: i16,
    suppress_score: i16,
    confidence_delta: i16,
    permanence: u8,
    polarity: i8,
    flags: u16,
    checksum: u32,
}
```

Allowed effects:

```text
reinforce accepted route support
suppress shortcut-specific false reading
replace previous anchor inside same intent
promote repeated correction operator
preserve unresolved conflict as WATCH
```

Forbidden effects:

```text
store raw turn text as cognitive memory
globally delete old route
let chat surface self-authorize memory writes
write unsupported claims into memory
claim nonlinear memory from record count or file size
```

## Write Operator

Every chat turn should produce one of these states:

```text
NO_WRITE
WATCH_TRACE
POSITIVE_DELTA
NEGATIVE_DELTA
CORRECTION_DELTA
CONFLICT_DELTA
```

Write gate:

```text
if verifier == PASS and evidence-bound:
  write positive route/support delta

if verifier blocks shortcut:
  write shortcut-specific anti-wave delta

if user correction is detected:
  write same-intent anchor replacement delta

if evidence conflicts:
  write WATCH conflict trace, not accepted memory

if unsupported/open-domain:
  no model-memory write, or only an unsupported-shortcut anti-lane
```

## Proof Fixture

The next code block is valid only if it proves:

```text
same prompt before memory:
  weak/review or lower score

turn/correction writes persistent wave delta:
  wave_memory_changed = true
  deltas_written > 0

same/follow-up prompt after reload:
  next_turn_used_persistent_memory = true
  accepted route score increases or false shortcut score decreases

unrelated route:
  unchanged within tolerance
```

Required JSON fields:

```json
{
  "memory_kind": "persistent_wave_delta",
  "wave_memory_changed": true,
  "deltas_written": 1,
  "field_before": {},
  "field_after": {},
  "next_turn_used_persistent_memory": true,
  "answer_changed_due_to_wave_memory": true,
  "unrelated_route_preserved": true,
  "claim_boundary": {
    "persistent_wave_memory_ready": true,
    "session_log_used_as_memory": false,
    "nonlinear_memory_proven": false,
    "general_llm_ready": false
  }
}
```

## Baselines

The implementation must compare against:

```text
no-memory next pass
raw session-log context replay
exact lexical lookup
route-only reinforcement
global route suppression
```

The wave-memory path wins only if it changes the intended relation state while
preserving nearby valid routes and avoiding unsupported claims.

## Failure States

```text
MEMORY_NO_DELTA
MEMORY_DELTA_NOT_USED_NEXT_PASS
SESSION_LOG_ONLY
ROUTE_KILL_SWITCH
UNRELATED_ROUTE_REGRESSION
UNSUPPORTED_WRITE
CLAIM_OVERREACH
```

Any of these must block `PERSISTENT_WAVE_MEMORY_READY`.

## First Implementation Target

Do not extend `linux_chat_v1` by adding a bigger `Vec<Turn>`.

Create a narrow module:

```text
src/llmwave_big/persistent_wave_memory.rs
```

Then wire Linux Chat V2 through it:

```text
linux-chat-v1 turn
  -> linux_profile reason/verifier
  -> persistent_wave_memory write gate
  -> durable wave delta store
  -> reload store
  -> next linux_profile field pass with memory overlay
  -> before/after eval
```

Only after that should the chat surface claim a durable memory effect.

