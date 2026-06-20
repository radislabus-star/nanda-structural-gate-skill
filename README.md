# NANDA Structural Gate Skill

NANDA Structural Gate is an experimental Codex skill for checking structural
relations before an agent finalizes an answer.

It is not a chatbot and not a replacement for retrieval, graph search, source
checking, or LLM reasoning. The goal is narrower:

```text
LLM extracts triads -> nanda-check verifies bindings -> agent answers with PASS/WATCH/VETO awareness
```

The current version includes a small deterministic Rust V0 checker. It is not
a full NANDA runtime yet, but it can already compare source triads against
candidate triads and return `PASS`, `WATCH`, or `VETO`.

For the planned cache-resident runtime, see
[`ARCHITECTURE_NANDA_6M.md`](ARCHITECTURE_NANDA_6M.md). NANDA-6M is specified
as a separate 6 MiB packed hot core, not as another command bolted onto the
current dynamic reference engine.

For the research line behind LLMWave, see
[`RESEARCH_DIRECTION.md`](RESEARCH_DIRECTION.md). It maps HRR, VSA,
Sparse Distributed Memory, Hopfield-style associative memory, superposition,
and Fourier/grokking work into concrete NANDA mechanisms and v47-v60
milestones.

For the word-birth and lexical-memory mechanism, see
[`LEXICAL_BIRTH_MECHANISM.md`](LEXICAL_BIRTH_MECHANISM.md). It fixes the staged
mechanism with literature footnotes: segmentation, fast mapping,
cross-situational convergence, usage/exemplar strengthening, grammar
integration, attractor cleanup, and anti-confusion.

## Why

LLMs can read individual facts correctly and still confuse the relation shape:

- supplier vs buyer;
- payment route vs delivery route;
- facts from two different deals spliced into one route;
- applicant vs manufacturer;
- contract party vs document holder;
- evidence for one claim attached to another claim.
- one evidence reference bound to incompatible role fillers.

This project tests whether a compact wave/VSA-style structural verifier can
catch those broken bindings cheaply enough to become a mandatory local gate.

## Trigger Rule

The gate should run when relation complexity crosses a threshold:

```text
complexity =
  entities
+ triads
+ 2 * routes
+ 2 * conflicting_sources
+ 3 * high_risk_role_swaps
```

The first mandatory threshold is:

```text
complexity >= 12
```

Small one-hop facts do not need the gate. Multi-party, multi-route, or
evidence-conflict tasks do.

## Repository Layout

```text
.
├── GOAL.md
├── ARCHITECTURE.md
├── PLAN.md
├── README.md
├── LICENSE
├── Cargo.toml
├── Cargo.lock
├── src/
│   └── main.rs
├── examples/
│   ├── triad-packet.example.json
│   ├── triad-packet.evidence-conflict.json
│   ├── triad-packet.interference-search.json
│   ├── triad-packet.interference-search-noisy.json
│   ├── triad-packet.interference-search-route-trap.json
│   ├── triad-packet.waw-code-runtime-trap.json
│   ├── triad-packet.waw-doc-owner-trap.json
│   ├── triad-packet.dataset-noise.json
│   ├── triad-packet.negative-shortcut-base.json
│   ├── triad-packet.negative-shortcut-lanes.json
│   ├── eval-corpus.json
│   ├── waw-corpus.json
│   ├── triad-packet.role-swap.json
│   ├── triad-packet.route-splice.json
│   ├── triad-packet.watch-low-complexity.json
│   ├── triad-packet.watch-missing-evidence.json
│   ├── route-trap.raw.txt
│   ├── self-dogfood.nanda.json
│   ├── triads.route-splice.md
│   ├── triads.code-flow.md
│   ├── triads.code-flow-splice.md
│   ├── triads.code-path-normalization.md
│   ├── triads.invariant-drift.md
│   ├── triads.linked-group-split.md
│   ├── triads.skill-flow.md
│   └── triads.skill-flow-splice.md
├── scripts/
│   ├── benchmark-v0.sh
│   ├── install-local.sh
│   ├── test-local.sh
│   ├── test-edge-cases.sh
│   └── sync-runtime.sh
└── nanda-structural-gate/
    ├── SKILL.md
    ├── agents/openai.yaml
    ├── references/
    │   ├── roadmap.md
    │   └── triad-packet.md
    └── scripts/
        ├── nanda-check
        ├── nanda-gate
        ├── nanda-init-task
        ├── nanda-pack-from-md
        ├── nanda-init-md
        ├── nanda-gate-md
        ├── nanda-split
        ├── nanda-split-md
        ├── nanda-report
        ├── nanda-comb
        ├── nanda-hgate
        ├── nanda-doctor
        ├── nanda-dogfood
        ├── nanda-eval
        ├── nanda-waw
        ├── nanda-extract
        ├── nanda-feedback
        ├── nanda-index
        ├── nanda-map
        ├── nanda-search
        ├── nanda-encode
        ├── nanda-decode
        ├── nanda-decode-eval
        ├── nanda-pattern-store
        ├── nanda-pattern-capacity
        ├── nanda-pattern-eval
        ├── nanda-pattern-bank
        ├── nanda-llmwave
        ├── nanda-llmwave-eval
        ├── nanda-llmwave-memory
        ├── nanda-llmwave-big
        ├── nanda-demo
        ├── nanda-cache
        ├── nanda-focus
        ├── nanda-proof
        ├── nanda-probe
        ├── nanda-dataset-doctor
        ├── nanda-aliases
        ├── nanda-budget
        ├── nanda-pack6m
        ├── nanda-bench6m
        ├── nanda-serve
        └── nanda-self-check
```

## Local Install

### Linux / macOS / WSL

Install the skill into the local Codex runtime and expose `nanda-check`:

```bash
scripts/install-local.sh
```

The installer builds the Rust binary and places it inside the runtime skill at
`nanda-structural-gate/bin/nanda`. The script names below are stable wrappers
around that binary.

### Windows PowerShell

From PowerShell:

```powershell
git clone https://github.com/radislabus-star/nanda-structural-gate-skill.git
cd nanda-structural-gate-skill
powershell -ExecutionPolicy Bypass -File .\scripts\install-windows.ps1
```

If `nanda-doctor.cmd` is not found after install, add this directory to your
user `PATH`:

```text
%USERPROFILE%\.local\bin
```

PowerShell one-liner:

```powershell
$bin = "$env:USERPROFILE\.local\bin"; [Environment]::SetEnvironmentVariable("Path", [Environment]::GetEnvironmentVariable("Path", "User") + ";$bin", "User")
```

Then open a new PowerShell window and run:

```powershell
nanda-doctor.cmd
nanda-search.cmd examples\triad-packet.interference-search-route-trap.json --input-format json --top-k 3
```

The Windows installer builds `target\release\nanda.exe`, copies the skill to
`%USERPROFILE%\.codex\skills\nanda-structural-gate`, and creates `.cmd`
wrappers in `%USERPROFILE%\.local\bin`.

Windows agents should use the generated `.cmd` wrappers, for example
`nanda-check.cmd`, `nanda-search.cmd`, and `nanda-doctor.cmd`, or call
`nanda.exe <subcommand>` directly.

Run the checker:

```bash
nanda --help
nanda-check
nanda-check --format json
nanda-check --triads examples/triad-packet.example.json
nanda-gate --triads examples/triad-packet.example.json
nanda-init-task --task-id live-check --domain contract --query "check routes"
nanda-pack-from-md examples/triads.route-splice.md --task-id md-splice --domain contract
nanda-init-md --task-id live-check --domain contract --query "check routes"
nanda-init-md --task-id code-check --template code --query "check source/runtime flow"
nanda-init-md --task-id skill-check --template skill --query "check source/runtime skill flow"
nanda-init-md --task-id project-check --template project --query "check repo readiness flow"
nanda-gate-md examples/triads.route-splice.md --task-id md-splice --domain contract
nanda-split examples/triad-packet.route-splice.json --by linked-group --out-dir split/
nanda-split-md examples/triads.watch-large.md --by group --out-dir split/
nanda-split-md examples/triads.code-flow-splice.md --by linked-group --out-dir split/
nanda-map examples/triads.code-flow-splice.md --domain code
nanda-hgate examples/triad-packet.hgate-size-only.json --input-format json
nanda-comb examples/triads.code-flow-splice.md --domain code --depth 2
mkdir -p .nanda
nanda-extract examples/route-trap.raw.txt --out .nanda/route-trap.json
nanda-index examples/triad-packet.interference-search-route-trap.json --input-format json --out .nanda/index.json
nanda-budget .nanda/index.json --input-format json
nanda-pack6m .nanda/index.json --input-format json
nanda-bench6m --replay-iterations 1000000 --projection-iterations 10000 --lane-iterations 1000000 --lane-sweep-iterations 100000
nanda-aliases examples/triad-packet.canonical-alias-pass.json --input-format json
nanda-cache build .nanda/index.json --input-format json --query "declaration requires protocols" --out-dir .nanda/cache
nanda-cache list .nanda/cache
nanda --version
nanda-focus .nanda/index.json --input-format json --query-file examples/triad-packet.interference-search-route-trap.json --query-format json --out .nanda/focus.json
nanda-proof .nanda/index.json --input-format json --query-file examples/triad-packet.interference-search-route-trap.json --query-format json --focus-out .nanda/focus.json --out .nanda/proof.json
nanda-proof .nanda/index.json --input-format json --query "declaration requires protocols" --fast
nanda-proof .nanda/index.json --input-format json --query "declaration requires protocols" --fast --cache-dir .nanda/cache
nanda-proof --cache-only .nanda/cache/<key>.manifest.json
nanda-search .nanda/index.json --input-format json --query-file examples/triad-packet.interference-search-route-trap.json --query-format json --top-k 3
nanda-search .nanda/focus.json --input-format json --top-k 3
nanda-search examples/triad-packet.interference-search.json --input-format json --top-k 3
nanda-search examples/triad-packet.interference-search-noisy.json --input-format json --format text
nanda-search examples/triad-packet.interference-search-route-trap.json --input-format json --top-k 3
nanda-encode --text "declaration requires protocols" --as-query-packet
nanda-decode examples/triad-packet.interference-search-route-trap.json --input-format json --top-k 5
nanda-decode examples/triad-packet.interference-search-route-trap.json --input-format json --top-k 3 --steps 3
nanda-decode examples/triad-packet.interference-search-route-trap.json --input-format json --top-k 3 --steps 3 --beam-width 3 --adaptive-scoring
nanda-decode-eval --suite examples/decode-corpus.json
nanda-pattern-capacity
nanda-pattern-eval --suite examples/pattern-learning-corpus.json
nanda-pattern-bank .nanda/index.json --input-format json --mode inspect
nanda-llmwave examples/triad-packet.interference-search-route-trap.json --input-format json --text "declaration requires protocols" --train
nanda-llmwave examples/triad-packet.interference-search-route-trap.json --input-format json --text "declaration requires protocols" --lens polarity
nanda-llmwave examples/triad-packet.interference-search-route-trap.json --input-format json --text "customs declaration requires" --lens token
nanda-llmwave examples/triad-packet.interference-search-route-trap.json --input-format json --text "declaration requires protocols" --lens convex
nanda-llmwave examples/triad-packet.interference-search-route-trap.json --input-format json --text "declaration requires protocols" --lens concave
nanda-llmwave examples/triad-packet.interference-search-route-trap.json --input-format json --text "declaration requires protocols" --lens prism
nanda-llmwave examples/triad-packet.interference-search-route-trap.json --input-format json --text "declaration requires protocols" --lens role
nanda-llmwave examples/triad-packet.interference-search-route-trap.json --input-format json --text "declaration requires protocols" --lens temporal
nanda-llmwave examples/triad-packet.interference-search-route-trap.json --input-format json --text "declaration requires protocols" --lens evidence
nanda-llmwave examples/triad-packet.interference-search-route-trap.json --input-format json --text "declaration requires protocols" --lens energy
nanda-llmwave examples/triad-packet.interference-search-route-trap.json --input-format json --text "declaration requires protocols" --lens anti
nanda-llmwave-memory write examples/triad-packet.interference-search-route-trap.json --input-format json --text "customs declaration requires payment" --out .nanda/llmwave-memory.json
nanda-llmwave-memory inspect .nanda/llmwave-memory.json
nanda-llmwave-memory vocabulary .nanda/llmwave-memory.json
nanda-llmwave-memory pack .nanda/llmwave-memory.json --out .nanda/llmwave-memory.llmw.bin
nanda-llmwave-memory unpack .nanda/llmwave-memory.llmw.bin
nanda-llmwave-memory retrieve .nanda/llmwave-memory.json --prefix "customs declaration requires"
nanda-llmwave-memory feedback .nanda/llmwave-memory.json --decision reject --token protocols --out .nanda/llmwave-memory-feedback.json
nanda-llmwave-memory correct .nanda/llmwave-memory.json --reject-token protocols --accept-token payment --out .nanda/llmwave-memory-corrected.json
nanda-llmwave-memory consolidate .nanda/llmwave-memory-feedback.json --out .nanda/llmwave-memory-consolidated.json
nanda-llmwave-memory decay .nanda/llmwave-memory-consolidated.json --factor 0.99 --out .nanda/llmwave-memory-decayed.json
nanda-llmwave-memory generate .nanda/llmwave-memory.json --prefix "customs declaration requires" --steps 2 --beam-width 2 --temperature 0
nanda-llmwave-memory chat .nanda/llmwave-memory.json --prompt "what does customs declaration require?" --steps 2
nanda-llmwave-memory answer .nanda/llmwave-memory.json --prompt "what does customs declaration require?" --facts 3
nanda-llmwave-memory answer .nanda/llmwave-memory.json --prompt "what does invoice issue?" --facts 3
nanda-llmwave-memory train corpus.txt --out .nanda/llmwave-text-memory.json
nanda-llmwave-memory grow .nanda/llmwave-memory.json examples/triad-packet.token-lens-business.json --input-format json --out .nanda/llmwave-grown.json
nanda-llmwave-memory eval --suite examples/llmwave-memory-corpus.json
nanda-llmwave-memory demo --corpus examples/llmwave-tiny-corpus.txt --prompt "what does customs declaration require?"
nanda-llmwave-memory density --counts 16,64,256,1024,4096 --facts 3
nanda-llmwave-big contract --format json
nanda-llmwave-big atlas --format json
nanda-llmwave-big active-core --format json
nanda-llmwave-big l2 --format json
nanda-llmwave-big word-birth --format json
nanda-llmwave-big surface-production --format json
nanda-llmwave-big surface-reconstruct --format json
nanda-llmwave-big surface-corpus-eval --format json
nanda-llmwave-big write --format json
nanda-llmwave-big consolidate --format json
nanda-llmwave-big eval --format json
nanda-llmwave-big query --text "supplier invoice payment customs" --format json
nanda-bench6m --mode active-core --support-build-iterations 1000 --format json
nanda-bench6m --mode write-density --support-build-iterations 1000 --format json
nanda-bench6m --mode consolidate --support-build-iterations 1000 --format json
nanda-llmwave-eval --suite examples/llmwave-corpus.json
nanda-llmwave-eval --suite examples/token-lens-corpus.json
nanda-demo examples/triad-packet.interference-search-route-trap.json --input-format json --text "declaration requires protocols"
nanda-demo --from-text examples/demo-task.raw.txt --task-id demo-raw --domain certification --text "declaration requires protocols"
nanda-demo --suite examples/demo-corpus.json --format json
nanda-search examples/triad-packet.source-weighting.json --input-format json --top-k 3
nanda-search examples/triad-packet.auto-query-memory.json --input-format json --query "lower operator debt route" --top-k 3
nanda-search examples/triad-packet.route-balanced-focus.json --input-format json --query "lower operator debt route" --route-cap 3 --route-triad-cap 1 --top-k 3
nanda-search examples/triad-packet.polarization-role-swap.json --input-format json --top-k 3
nanda-search examples/triad-packet.polarization-reversed-stop.json --input-format json --top-k 3
nanda-probe examples/triad-packet.negative-shortcut-lanes.json --input-format json --top-k 3
nanda-probe --suite examples/probe-corpus.json --input-format json --top-k 3
nanda-search examples/triad-packet.interference-search-route-trap.json --input-format json --top-k 3 > .nanda/search.json
nanda-feedback .nanda/search.json --decision accept --note "accepted focused peak"
nanda-eval --case examples/triad-packet.interference-search-route-trap.json:certification:FOCUSED --case examples/triad-packet.interference-search-noisy.json:certification:WATCH
nanda-eval --suite examples/eval-corpus.json
printf '{"command":"doctor"}\n' | nanda-serve
printf '{"command":"proof_cache_only","manifest":".nanda/cache/<key>.manifest.json"}\n' | nanda-serve
printf '{"command":"proof_cache_only","manifest":".nanda/cache/<key>.manifest.json","response":"compact"}\n' | nanda-serve
printf '{"command":"llmwave_token","input":"examples/triad-packet.interference-search-route-trap.json","text":"customs declaration requires"}\n' | nanda-serve
printf '{"command":"llmwave_chat","memory":".nanda/llmwave-memory.json","prompt":"what does customs declaration require?","steps":2}\n' | nanda-serve
printf '{"command":"llmwave_answer","memory":".nanda/llmwave-memory.json","prompt":"what does customs declaration require?","facts":3}\n' | nanda-serve
nanda-doctor
nanda-dogfood .
nanda-map-code src/main.rs
nanda-dogfood . --refactor-plan --format json
nanda-self-check
```

CLI exit codes:

```text
0 - PASS
1 - VETO
2 - ERROR
3 - WATCH
```

`nanda-gate` is the stricter pipeline wrapper. It allows only `PASS`.

`nanda-pack-from-md` converts Markdown triad tables into JSON packets, so agents
can work in readable `.md` first and only generate JSON at the gate boundary.
`nanda-init-md` and `nanda-gate-md` are the preferred live-agent workflow.
For code architecture, use `nanda-init-md --template code`.
For Codex skill workflows, use `--template skill`.
For repository readiness summaries, use `--template project`.
For machine flows, prefer `nanda-split <packet.json> --by linked-group`; it
writes JSON packets that can go directly into `nanda-check --triads`.
For manual worksheets, use `nanda-split-md --by group`, `--by route`, or
`--by linked-group`. Prefer `linked-group` when source groups and candidate
groups use different names; it pairs source rows with candidate rows using the
`nanda-map` group links and triad-level exact matches.
For code projects, add `--normalize-paths` to collapse paths such as
`src/bin/check.rs` into `bin::check` and `src/core/gate.rs` into `core::gate`.
`nanda-self-check` verifies the checker/gate/tooling contour itself.
`nanda-dogfood .` verifies a repository against its own
`examples/self-dogfood.nanda.json` packet. It is the fast agent-facing readiness
check: a root `WATCH` is acceptable only when it is size-only, the map is clean,
and every linked branch is `PASS`.
`nanda-map-code` is the refactor planning pass for large Rust files. It clusters
functions, reports cross-cluster dependencies, suggests target files, and marks
extraction risk. Use `nanda-dogfood . --refactor-plan` when you want the normal
structural verdict plus code-boundary recommendations in one packet.
`nanda-report` is agent-first: it returns a JSON decision packet by default.
Use `--format md` only when a human-facing report is explicitly needed.
`nanda-map` exposes the core structural map: source/candidate group sizes,
interference matrix, dominant source group, mixed candidate groups, and repair
tasks.
`nanda-hgate` is the hierarchical gate for large packets. It runs one
global map/check, splits by linked group, runs local gates, and returns
`STRUCTURALLY_ACCEPTED` only when the global `WATCH` is size-only and every
local branch is `PASS`. If `foreign_pull`, conflicts, or any local `VETO`
exist, it returns `REPAIR_REQUIRED`.
`nanda-extract` converts simple arrow text into a triad packet. The supported
line format is `subject -> relation -> object [route=x group=y ...]`, with
`## triads` and `## candidate_triads` sections.
`nanda-index` builds a reusable memory packet from one or more triad packets or
Markdown worksheets.
`nanda-search` is the memory-index retrieval surface. It treats `triads`
as memory and either the same packet's `candidate_triads` or a separate
`--query-file` as the partial query, then returns top-k route/group peaks with
support, foreign pulls, missing edges, source weights, destructive
interference, constructive interference, route-balanced focus, coarse-to-fine
local paths, polarization, and an answer projection. If no `candidate_triads`
exist, text from `--query` or packet `query` is converted into lightweight
`auto_query_triads`; check `query.source` before trusting the peak.
`nanda-dataset-doctor` is the corpus-quality gate. Run it before search on
large memory packets; it warns about route imbalance, hub dominance, duplicate
CURRENT facts, oversized direct-search packets, and weak text-only queries.
`nanda-focus` is the v24 focused packet builder. It takes a large memory packet
plus `candidate_triads`, `--query-file`, or text `--query`, selects a
route-balanced window with `--max-triads` defaulting to the 15,000-triad hot
proof cap, and writes a smaller JSON packet that can be passed to
`nanda-search`, `nanda-budget`, `nanda-pack6m`, or `nanda-hgate`.
`nanda-cache build` is the v64 reusable focus-cache builder. It stores a
focused packet under a key derived from corpus content, query text/source, and
focus caps. Use it before repeated large-corpus `nanda-proof --fast` queries.
Use `nanda-cache list .nanda/cache` to inspect available focused packets.
`nanda-proof` is the v27 one-shot proof pipeline. It runs corpus diagnostics,
builds the focused packet, checks the NANDA-6M runtime contract, runs
interference search, runs the packed bridge, and returns `ANSWER_READY`,
`WATCH`, or `VETO`. `ANSWER_READY` requires both the retrieval field and the
packed peak to be safe; otherwise the output is an explainable review report.
Inspect top-level `reason_codes`, `proof_confidence`, and `proof_compare`
before trusting a peak. Use `nanda-proof --suite examples/proof-corpus.json`
to run the proof regression corpus.
Use `nanda-proof --fast` for large-corpus agent loops where full raw search is
too expensive. Fast proof still runs corpus diagnostics, focused search, and
packed proof, but explicitly marks `raw_search_summary.skipped=true`, adds
`RAW_SEARCH_SKIPPED`, and reports `proof_compare.state` as
`FOCUSED_PACKED_ALIGNED` or `FOCUSED_ONLY_REVIEW` instead of pretending the
full raw/focused compare ran.
Use `nanda-proof --cache-dir .nanda/cache` to reuse a focused packet created by
`nanda-cache build`; inspect `focus_cache.state`. `CACHE_HIT` means the focus
window was reused, `CACHE_MISS` means it was rebuilt in memory, and
`CACHE_WRITTEN` appears only when `--write-cache` is passed.
Use `nanda-proof --cache-only <manifest-or-single-manifest-dir>` when the agent
must avoid loading the original large corpus. It runs focused search and packed
proof from the cached focused packet, sets `proof_mode=cache-only-focused`, and
marks `corpus.state=CORPUS_NOT_LOADED`.
`nanda-search` now emits `resonant_field`, the v28 physical field layer. It
checks phase lock, standing-wave reflection, route-boundary leakage,
destructive locality, multiscale agreement, energy conservation,
frequency/mode scan, temporal phase, coherence memory, and the final
`waw_status`. Treat `WAW_RESONANCE` as a strong field phenomenon, but still
require proof/packed gates before final `ANSWER_READY`.
`nanda-encode` is the v33 text-to-wave bridge. It tokenizes text, projects
tokens into deterministic position-bound waves, superposes them into a
1024-dimensional field signature, and can emit preview candidate triads with
`--as-query-packet`. Use it when a raw phrase needs to enter `nanda-search` or
`nanda-decode` without pretending that text and structural triads are already
the same representation.
`nanda-decode` is the first LLMWave bridge. It runs the same interference
field as `nanda-search`, then decodes the top field into ranked
`next_structural_pattern` candidates. It does not generate prose yet; it emits
candidate `subject -> relation -> object` continuations with route, role,
polarity, continuity, and support scores. With `--steps N`, it recurrently
feeds the selected pattern back into the query context and re-runs the field.
`PATTERN_SATURATED` means the current field has no new structural continuation
under the selected window.
`nanda-decode-eval` is the regression surface for the decoder. It checks
expected decoder state, top structural pattern, recurrent final state, minimum
completed recurrent steps, and optional v42 beam trajectory checks before
trusting LLMWave changes. With `--beam-width N`, `nanda-decode` keeps multiple
structural continuations in superposition and reports ranked trajectories. With
`--adaptive-scoring`, v45 field-state-aware weights are reported under
`adaptive_pattern_scoring`.
`nanda-feedback` can also read `nanda-decode` output. In that mode it emits
`continuation_memory`: accepted decoded patterns are reinforced during future
decode ranking, rejected decoded patterns are suppressed locally, and WATCH
patterns remain review evidence. v35-v60 compact this into a 32-byte pattern
store, replay it during decode, estimate capacity, expose shortcut-specific
negative continuation lanes, run an `nanda-llmwave` read/write/retrieve proof loop,
and report the NANDA-6M pattern runtime contract. `nanda-pattern-eval` measures
the actual learning effect: baseline decode -> feedback memory -> trained
decode, with checks for top-pattern movement or score reinforcement.
`nanda-pattern-bank` now builds or inspects that learned continuation layer as
a cleanup-memory bank. `nanda-llmwave` reports v47 HRR binding, v48 cleanup,
v49 attractor energy, v50 capacity, v51 anti-wave audit, v54 packed HRR,
v55 cleanup thresholds, v56 anti-wave locality, v57 capacity baseline,
v58 hot-cycle readiness, v59 proof summary, the v60 public demo packet, and
the v67 `llmwave_contract` Field + Lens readout. v68-v75 add Token Lens:
token-pattern records, position-phase prefix waves, next-token resonance,
token cleanup, shortcut-specific token anti-wave, and a token eval corpus. Use
`--lens pattern`, `--lens polarity`, `--lens cleanup`, or `--lens token` to
inspect the active lens. v76-v80 add the first LLMWave optics core:
`lens_taxonomy`, repeatable `field_snapshot`, Convex Lens for gathering aligned
weak signals into a route basin, Concave Lens for splitting contested peaks,
and Prism Lens for explaining route/relation/role/polarity contributions. Use
`--lens convex`, `--lens concave`, or `--lens prism` when the question is not
"what is the top answer?" but "how did the field form this peak?" v81-v85 add
semantic optics: Role Lens for actor/action/target binding, Temporal Lens for
recurrent order and route jumps, Evidence Lens for support binding, Energy Lens
for basin stability, and Anti Lens for destructive-interference reports. Use
`--lens role`, `--lens temporal`, `--lens evidence`, `--lens energy`, or
`--lens anti` when the field needs meaning-axis inspection. v86-v95 add
LLMWave Memory Core through `nanda-llmwave-memory`: `write` creates one
wave-memory object from triads/token/phrase continuations, `retrieve` reads
next-token candidates through resonance, `feedback` applies accept/reject/WATCH
learning, `consolidate` merges duplicate continuations, `decay` forgets weak
records, `generate` runs recurrent retrieval, and `eval` checks memory behavior
against `examples/llmwave-memory-corpus.json`. v96-v104 add the first generator
surface: `vocabulary`, deterministic/temperature sampler metadata, beam
candidates, semantic decoder text, `chat`, text training, memory growth, and
self-correction through `correct`. v105-v109 add model-core checks: `inspect`
reports memory file format, schema hash, tokenizer contract, and model config;
`pack`/`unpack` write and validate the first binary `.llmw.bin` prototype; the
quality eval now covers direct retrieve, feedback shift, text training, memory
growth, and decay. v127-v157 add the density research path: useful recall and
reversed-trap checks, lexical/relation/vector baselines, phase-lock,
noise-pressure, packed hot-loop proxy, focus-window experiment, fixed-basis
and margin-erosion checks, anti-wave lift candidate, L2 prefix contour,
L3-to-L2 rerank contract, final nonlinear-density verdict, adversarial stress
metadata, baseline duel, anti-wave ablation proxy, useful-capacity score, and
`nanda-bench6m --mode density`. Treat these as measurements and guardrails, not
proof that nonlinear density is solved. Treat
`LLMWAVE_LENS_READY` as a usable structural readout and
`LLMWAVE_LENS_REVIEW` / `LLMWAVE_LENS_WATCH` as unresolved.
`nanda-llmwave-eval` verifies those fields through `examples/llmwave-corpus.json`.
`nanda-llmwave-big contract` starts the v158-v160 LLMWave-Big track. It defines
the Big Model Contract, bigness metrics, L2/L3 boundaries, and claim firewall.
It deliberately reports `BIG_MODEL_NOT_PROVEN`: this is the contract and
measurement surface for the future Wave Atlas + small Active Core, not a claim
that an LLM or nonlinear memory has already been proven.
`nanda-llmwave-big atlas` adds the v161-v170 Wave Atlas contract: fixed
Symbol/Operator/Schema/Residual records, cold evidence refs, domain banks,
indexes, Atlas Doctor checks, and a loader preview that outputs compact IDs,
negative lanes, and evidence refs without evidence text or JSON in the active
packet.
`nanda-llmwave-big active-core` adds the v171-v180 Active Core contract and a
typed sample cycle: fixed ActivePacket records, a 6,291,456-byte budget,
schema/residual wave projection, focus competition axes, runtime operation
list, and an `ACTIVE_CORE_READY` sample verdict. `nanda-bench6m --mode
active-core` measures that typed in-memory microkernel and excludes CLI, JSON,
Atlas loading, and report serialization.
`nanda-llmwave-big l2` adds the v181-v190 L2 Word Field contract: active
surface slice, prefix wave, 128-4096 candidate cache, L3 bias into surface
candidates, anti-wave suppression for schema-breaking prefix matches, L2/L3
sync policy, multilingual surface banks, and L2 eval metrics.
`nanda-llmwave-big word-birth` adds the v246-v252 lexical birth mechanism from
the literature line: statistical segmentation, fast mapping, cross-situational
convergence, usage/exemplar strengthening, grammar integration, attractor
cleanup, and anti-confusion. It reports fixed 32-byte candidate and binding
records plus a strict claim boundary: this is a mechanism contract, not proof
that a real corpus has learned new words. Surface text is produced from
grapheme/byte atoms, morpheme atoms, surface programs, and exact evidence-copy
spans; words are not modeled as a flat numeric-handle-to-string lookup. The
canonical mechanism note is [`LEXICAL_BIRTH_MECHANISM.md`](LEXICAL_BIRTH_MECHANISM.md).

`nanda-llmwave-big surface-production` adds the v253-v260 surface production
memory layer. It fixes the concrete record boundary for producing visible forms:
`SurfaceAtom16`, `SurfaceProgram32`, `EvidenceCopySpan24`, and
`SurfaceProductionCandidate32`. The hot core scores compact program ids and copy
span refs; UTF-8 materialization stays outside the hot loop. Common forms are
composed from atoms and morphemes, while rare names/codes use exact evidence
copy spans. The report keeps `real_corpus_trained`,
`free_form_spelling_proven`, and `nonlinear_surface_memory_proven` false until
evals prove those claims.

`nanda-llmwave-big surface-reconstruct` adds the v261-v270 cold materializer and
toy reconstruction eval. It expands `SurfaceProgram32` through `SurfaceAtom16`,
copies exact rare forms through `EvidenceCopySpan24`, and uses byte fallback for
unknown forms. Its report includes `exact_match_rate`, `copy_error_rate`,
`fallback_rate`, `program_reuse_ratio`, `bytes_per_reconstructable_surface`,
and direct-lookup baseline bytes. The current state is deliberately
`TOY_RECONSTRUCTION_PASS_NOT_DENSITY_PROOF`.

`nanda-llmwave-big surface-corpus-eval` adds the v271-v280 corpus-scale surface
memory eval. It compares direct lookup, per-form `SurfaceProgram32`, byte-only
fallback, and family-template reuse. The report introduces `SurfaceFamily32` and
`SurfaceBinding8` as the first measurable route toward combinatorial surface
memory: shared roots and suffixes can generate many virtual forms. Its verdict
is deliberately `SURFACE_DENSITY_CANDIDATE_NOT_PROVEN`: useful density is
visible on the synthetic suite, but real corpus training and nonlinear surface
memory proof remain false.

`nanda-llmwave-big write` adds the v191-v205 Schema/Residual Write contract:
write decomposition, reconstructability score, centroid update plus residual
decision, Residual V1 format, anti-residual, promotion/split rules, ablation,
source-aware weighting, and a write-density microbenchmark. It reports
`RESIDUAL_SAVING`, not nonlinear memory proof.
`nanda-llmwave-big consolidate` adds the v206-v218 consolidation/sleep contract:
duplicate merge, alias merge, conflict preservation, schema strength, weak
residual decay, anti-memory, before/after eval, Atlas rebuild, memory-bank
repacking, cognitive compression score, and `nanda-bench6m --mode
consolidate`.
`nanda-llmwave-big eval` adds the v219-v230 Big Cognition Eval surface:
documents/money/goods/certification/code/config/source/route domains,
inference, role-swap, contradiction, multi-hop, missing-evidence, generation,
style, code, and business task families. It can report `COGNITIVE_LIFT` while
still keeping LLM, nonlinear-memory, cache-only, and final-candidate claims
false.
`nanda-llmwave-big query` adds the v231-v245 runtime product surface: local
daemon contract, skill integration command, editor typing mode, code-review and
business-graph modes, memory import/export, personal/domain Atlas, contested
field safety, explainability fields, performance target, 1M-fact load-test
contract, release-candidate checklist, and v1-candidate criteria.
`nanda-demo` is the v62 weak-spot surface: it compresses the v60 JSON into a
short state/top-pattern/proof/signals/weak-spots report for humans and agents.
It can also start from raw relation notes via `--from-text`: explicit
`subject -> relation -> object [route=x group=y]` lines become a temporary demo
packet, while free-text fallback is marked review-only.
`examples/demo-corpus.json` keeps three demo modes honest: proof-ready,
anti-wave-visible, and review-only.

Recommended agent rule:

```text
relation-heavy task
  -> extract/build triad packet, or use nanda-demo --from-text for raw relation notes
  -> nanda-check / nanda-search / nanda-proof as needed
  -> if using an LLMWave peak as support, run nanda-demo
  -> answer only when state=PUBLIC_DEMO_READY and weak_spots=[]
```

If `nanda-demo` returns `PUBLIC_DEMO_REVIEW` or any weak spot, treat the peak
as a repair hint rather than an answer route.
`nanda-feedback` also records v29 `resonance_memory`: the accepted, rejected,
or watched shape of the field itself. It stores the peak, route, relation,
role mode, WAW status, phase/standing-wave/energy/boundary states, and compact
support/anti terms. `nanda-index` merges these forms, and later
`nanda-search` softly replays them as `resonance_memory`: accepted forms
reinforce similar peaks and rejected forms suppress similar bad field shapes.
This is a learned interference hint, not a standalone PASS.
`nanda-aliases` is the explicit canonicalization diagnostic. If a JSON packet
contains `aliases`, NANDA applies exact high-confidence variants to `subject`,
`object`, `route`, and `group` before check/map/search/pack6m. It does not
guess aliases automatically. Ambiguous or low-confidence aliases return WATCH
and are shown in `canonicalization.conflicts` or
`canonicalization.warnings`.
`nanda-budget` is the NANDA-6M Phase 1 planner. It does not run the packed hot
core yet; it checks both the broad 6 MiB arena layout and the v24 focused
15,000-triad proof runtime. It returns `FITS_L3`, `FOCUS_REQUIRED`,
`SPLIT_REQUIRED`, or `SPILL_REQUIRED`; inspect `runtime_focus` and
`safe_for_hot_core` before treating the packet as hot-runnable.
`nanda-pack6m` is the first cold-to-hot bridge. It builds deterministic
dictionaries and sample `PackedTriad32` records from a packet, proving that the
string/JSON world can be reduced to the planned fixed records before packed
interference search exists. It also reports a fixed 1024-dimensional packed
candidate/query projection wave summary, memory/source route/group centroid
summaries, and a first packed candidate-query-vs-memory-centroid peak score.
Inspect `peak_decision.safe_to_answer`: `PACKED_THIN` means the packed field
has a weak honest peak, not a trustworthy answer route yet.
Inspect `packed_support`: it explains the packed peak as supporting and
anti-supporting memory records. A thin peak with a small positive `net_dot`
usually means constructive and destructive contributions nearly cancelled.
Inspect `packed_lanes`: it is a preview-only `PackedLane64` bridge that masks
the current anti-support records and reports the possible `net_dot` change
before learned lanes are applied in the hot loop.
Inspect `packed_lane_keys`: it is the cold stable lane signature. The key is
compiled into current-window `PackedLane64` masks; record masks are not treated
as persistent memory.
Inspect `packed_lane_store`: it reports the hot compiled lane arena cost. Each
stored runtime lane is 64 bytes, so the 1 MiB arena holds 16,384 compiled lanes.
Inspect `runtime_contract`: it is the v20 hot attach gate. `PACKED_RUNTIME_READY`
means the focused packet fits the current `PackedHotCore` workspace contract.
In v24 the active proof window is intentionally fixed at 15,000 triads with 64
default field requests. `FOCUS_REQUIRED` means the packet may fit the broad
65,536-record triad arena, but it is too wide for one 15k hot proof. Do not
silently spill that case into RAM; build a `nanda-focus` packet or split the
packet first.
Inspect `packed_lane_replay`: it matches feedback shortcuts against current
stable lane keys, compiles matched keys into current-window `PackedLane64`
masks, and reports replayed `before_net_dot -> after_net_dot`.
In v3.0 it also reports an observer-to-compute sweep: observer, soft, medium,
and full touch strengths. This makes replay a controlled computational
intervention diagnostic: it can show whether feedback would stabilize the
packed field, but it still cannot set `safe_to_answer=true` by itself.
Inspect `packed_replay_decision`: it compares the raw `peak_decision` with the
replay-adjusted field and returns a firewall verdict such as
`STABLE_WITH_REPLAY`, `REPLAY_RESCUED_THIN_FIELD`, or
`REPLAY_DESTABILIZED_FIELD`. The firewall may move a thin field to review-ready,
but it still blocks direct answer permission. Its decision is now produced by
the hot-compatible typed core function `nanda_6m::evaluate_replay`; JSON is only
the outer report bridge.
Inspect `packed_lane_application`: it runs a single applied lane pass over the
support-map. `PACKED_LANE_FOCUSED_CANDIDATE` means the lane-adjusted field is
ready for a real hot-loop implementation, but it still keeps
`safe_to_answer=false`.
`nanda-bench6m` is the hot-core microbenchmark. It intentionally excludes CLI
startup, JSON parsing, dictionary packing, file I/O, and report serialization.
It measures the typed replay firewall (`evaluate_replay`), the in-memory packed
1024-dimensional projection/centroid scoring path, and packed lane
compile/application. It also measures unordered lane-arena sweep, aligned
field/lane sweep, fused aligned compile+sweep, support-field building from
packed triads, score-cached support-field building, compile+sweep variants, and
the v20 `PackedHotCore` runtime contract around the full hot-cycle.
Use it when you need real kernel timing rather than wrapper timing:

```bash
nanda-bench6m --format text
nanda-bench6m --mode replay --replay-iterations 5000000 --format json
nanda-bench6m --mode projection --projection-iterations 20000 --triads 64
nanda-bench6m --mode lane --lane-iterations 5000000 --format json
nanda-bench6m --mode lane-sweep --lane-sweep-iterations 1000000 --lane-sweep-width 64 --format json
nanda-bench6m --mode aligned-lane-sweep --lane-sweep-iterations 1000000 --lane-sweep-width 64 --format json
nanda-bench6m --mode aligned-compile-sweep --lane-sweep-iterations 1000000 --lane-sweep-width 64 --format json
nanda-bench6m --mode support-build --support-build-iterations 1000 --lane-sweep-width 64 --triads 64 --format json
nanda-bench6m --mode support-build-compile-sweep --support-build-iterations 1000 --lane-sweep-width 64 --triads 64 --format json
nanda-bench6m --mode support-score-build --support-build-iterations 1000 --lane-sweep-width 64 --triads 64 --format json
nanda-bench6m --mode support-score-build-compile-sweep --support-build-iterations 1000 --lane-sweep-width 64 --triads 64 --format json
nanda-bench6m --mode support-bucket-build --support-build-iterations 1000 --lane-sweep-width 64 --triads 64 --format json
nanda-bench6m --mode support-bucket-build-compile-sweep --support-build-iterations 1000 --lane-sweep-width 64 --triads 64 --format json
nanda-bench6m --mode hot-cycle --support-build-iterations 1000 --lane-sweep-width 64 --triads 64 --format json
nanda-bench6m --mode density --support-build-iterations 1000 --triads 15000 --format json
```
`nanda-serve` is the JSONL agent API. It keeps one process alive and accepts
requests such as `{"command":"doctor"}`, `{"command":"check","packet":...}`,
`{"command":"search","packet":...}`, or
`{"command":"proof_cache_only","manifest":"..."}`. For cache-only proof, the
server keeps the focused packet in process memory after the first request and
reuses repeated proof results for the same manifest/options. Inspect
`serve_cache.state`. Use `response:"compact"` when the agent only needs the
verdict, peak, confidence, reason codes, and cache state.
`nanda-feedback` is the feedback-memory surface. It records whether a search
peak or decoded continuation was accepted, rejected, or kept under WATCH,
together with margin, support ids, anti ids, and a compact memory patch. Reject
feedback emits
`negative_shortcuts`; accept feedback emits `positive_shortcuts`; all decisions
emit `resonance_memory` so the honest or dishonest field form can be recognized
again. `nanda-index` can carry these into future search, so rejected shortcuts
are suppressed, accepted routes are constructively reinforced, and matching
resonance forms are softly replayed without granting automatic safety.
Decode feedback emits `continuation_memory`; `nanda-index` carries it into
future decode runs, where it shifts pattern scores before recurrent selection.
`nanda-probe` compares the same search before and after negative lanes. Use it
before claiming destructive interference helped. `nanda probe --suite` runs a
probe regression corpus. `SHIFTED_TO_REVIEW` means the false shortcut moved,
but the replacement peak is still a review target rather than a final answer.
`nanda-eval` is the regression surface. It checks expected peak/state pairs
from `--case` or `--suite`, so interference changes are measured before they
are trusted.
`nanda-waw` is the WAW benchmark surface. It checks hard trap cases where a
plain lexical baseline selects the wrong route, while the interference peak
selects the structurally connected route and explains the centroid drift.
`nanda-doctor` is the release smoke test. It runs built-in focused/noisy
interference checks without needing repository example files.

Self-check modes:

```bash
nanda-self-check
NANDA_SELF_CHECK_RUNTIME_ONLY=1 nanda-self-check
nanda-dogfood .
nanda-dogfood . --format json --out-dir .nanda/
nanda-report --overall overall.md --route invoice:invoice.md
nanda-report --format md --overall overall.md --route invoice:invoice.md
```

Agent decision packet fields:

```text
action
safe_to_draft
safe_to_send
blocking
review_required
repair_prompts
next_prompt
```

Core version fields:

```text
nanda --version
core_version: sparse-triad-v6.0-llmwave-proof
wave_dim: 1024
```

`v3.3-modular-router` keeps recursive topology combing, structural peak search,
reusable memory indexes, arrow-text extraction, feedback packets, regression
evaluation, release doctor checks, eval corpus loading, JSONL serve mode, and
richer field interpretation. It adds a WAW corpus where the lexical baseline is
expected to pick the wrong route and the interference field must recover the
connected structure. It also adds dataset immunity: before searching a large
corpus, check whether the field is route-imbalanced, hub-heavy, duplicated, or
too weakly queried. Source weighting makes current/canonical evidence pull
harder than archive/noise evidence. Auto-query triads make text-only search
less blind. Learning negative lanes add destructive interference for rejected
shortcut peaks and strengthen repeated rejects. Route-balanced focus reduces
large/noisy corpora before direct search. Coarse-to-fine output shows the local
supporting path after the coarse peak. Polarization adds role-direction lanes
so reversed structures can look lexically similar but resonate differently. The
field-state machine then converts measured signals into an agent-safe action:
answer from support, split/query more, focus the corpus, or stop for polarity
repair. Local negative lanes make destructive interference route/group-aware
and suppress the rejected reading shape, not just a whole peak name. Probe
reports compare before/after search so the agent can verify whether a negative
lane really helped or merely moved uncertainty elsewhere. The
feedback-memory layer adds positive lanes: accepted peaks become constructive
reinforcement for the same route/group/support shape in later search. The
hierarchical gate makes large packets usable without pretending one flat PASS
is enough: a global size-only `WATCH` can become `STRUCTURALLY_ACCEPTED` only
after every linked local branch passes.
`POLARITY_REVERSED` gate prevents a reversed top peak from being used as an
answer route. The search path is intentionally small and universal: encode
triads as slot-bound waves, superpose a partial query, score memory
routes/groups by weighted and polarized interference, then interpret, record,
test, and smoke-check the top peaks.
If `foreign_pull` is non-empty, strict gate output is not `PASS`; repair the
named candidate triads or split the route first.

Interference search output:

```text
peak
score
verdict
field_state
safe_to_answer
top_peak
peak_margin
lexical_baseline
wins_over_lexical_baseline
peak_decision.state
peak_decision.safe_to_answer
field_state_machine.state
field_state_machine.safe_to_answer
field_state_machine.action
field_state_machine.blocking
field_interpretation.state
field_interpretation.lexical_trap_detected
field_interpretation.centroid_drift
field_interpretation.corpus
destructive_interference
constructive_interference
propagation.component_score
center
supporting_triads
anti_triads
missing_edges
answer_projection
```

Read the margin conservatively. A large margin means the structural route is
focused; a small margin means the peak is useful but should be treated as a
WATCH-like retrieval hint until evidence is checked.
Use `peak_decision` as the agent-facing trust layer. `FOCUSED` means the peak
has enough margin and component strength to draft from; `WATCH` means retrieve
more evidence or ask the LLM to inspect support/anti-triads before answering.
Use `field_state_machine` as the stricter action layer. `FIELD_FOCUSED` and
`FIELD_SAFE` can be drafted from support. `FIELD_CONTESTED`, `FIELD_THIN`, and
`FIELD_NOISY` are retrieval hints. `FIELD_REVERSED` is a hard stop.
The strongest regression example is `triad-packet.interference-search-route-trap.json`:
the lexical baseline selects the `customs` route, while the interference peak
selects the connected `certification` route. The important signal is that
`certification` covers the query through one connected component, while
`customs` is split into a lexical trap plus a separate customs chain.

Dogfood agent output:

```text
ACTION: SAFE_TO_EDIT | SPLIT_REQUIRED | REPAIR_REQUIRED | REVIEW_REQUIRED
ROOT: PASS | WATCH size-only | VETO
STRUCTURE: foreign_pull=N invariant_violation=N
BRANCHES: X/Y PASS
SAFE_TO_EDIT: true | false
```

## Recommended Workflow

Use this workflow for large code-flow, repository, contract, logistics, or
multi-route graphs:

```bash
mkdir -p .nanda
nanda-dogfood . --out-dir .nanda/
nanda-extract notes.raw.txt --out .nanda/notes.json
nanda-index memory-a.json memory-b.md --out .nanda/index.json
nanda-dataset-doctor .nanda/index.json --input-format json
nanda-focus .nanda/index.json --input-format json --query-file query.json --query-format json --out .nanda/focus.json
nanda-proof .nanda/index.json --input-format json --query-file query.json --query-format json --focus-out .nanda/focus.json --out .nanda/proof.json
nanda-proof --suite examples/proof-corpus.json --input-format json
nanda-hgate big-flow.json --input-format json --by linked-group
nanda-search .nanda/focus.json --input-format json --top-k 5
nanda-search .nanda/index.json --input-format json --query "lower operator debt route" --route-cap 256 --route-triad-cap 32 --top-k 5
nanda-feedback .nanda/search.json --decision accept --note "accepted route" --out .nanda/accept.json
nanda-index memory-a.json .nanda/accept.json --out .nanda/index-with-positive-lanes.json
nanda-search .nanda/index-with-positive-lanes.json --input-format json --query-file query.json --query-format json --top-k 5
nanda-feedback .nanda/search.json --decision reject --note "customs shortcut" --out .nanda/reject.json
nanda-index memory-a.json .nanda/reject.json --out .nanda/index-with-negative-lanes.json
nanda-probe .nanda/index-with-negative-lanes.json --input-format json --top-k 5
nanda-eval --case route-trap.json:certification:FOCUSED --case noisy.json:certification:WATCH
nanda-eval --suite examples/eval-corpus.json
nanda-waw --suite examples/waw-corpus.json
printf '{"command":"doctor"}\n' | nanda-serve
nanda-doctor
nanda-comb big-flow.json --input-format json --depth 2 --out-dir comb/
nanda-map big-flow.md --domain code --normalize-paths
nanda-gate-md big-flow.md --domain code --normalize-paths --format json
nanda-split big-flow.json --input-format json --by linked-group --out-dir split-json/
nanda-split-md big-flow.md --by linked-group --normalize-paths --out-dir split/
for f in split/*.md; do
  nanda-gate-md "$f" --domain code --normalize-paths --format json
done
```

Interpretation:

- prefer `nanda-comb --depth 2` for machine workflows;
- prefer `nanda-hgate` when a large packet gets global WATCH because of size;
- run `nanda-map` first to inspect `mixed_candidate_groups` and `foreign_pull`;
- treat non-empty `foreign_pull` as a repair stop;
- do not force one global PASS when the graph exceeds size limits;
- use `linked-group` split to produce paired source/candidate worksheets;
- read `STRUCTURALLY_ACCEPTED` as global size-only WATCH plus local PASS, not
  as a flat global PASS.
- accept only route-level PASS for each paired worksheet;
- if the global graph is clean but too large, report it as size-stopped, not as
  structurally failed.
- use `comb_tree` when you need the full topology plus recursive branch record.

Validated code-flow pattern:

```text
global map: clean
global strict gate: size stop
linked split: paired worksheets
local gates: PASS per pair
```

Run local tests:

```bash
scripts/test-local.sh
scripts/benchmark-v0.sh
scripts/test-edge-cases.sh
```

## Release

Current release: `v3.3.0`.

Release notes are maintained in [CHANGELOG.md](CHANGELOG.md). Before tagging a
release, run:

```bash
scripts/test-local.sh
scripts/test-edge-cases.sh
scripts/benchmark-v0.sh
nanda-doctor
nanda-eval --suite examples/eval-corpus.json
nanda-waw --suite examples/waw-corpus.json
nanda-dataset-doctor examples/triad-packet.dataset-noise.json --input-format json --route-cap 8 || test "$?" -eq 3
nanda-search examples/triad-packet.negative-shortcut-lanes.json --input-format json --top-k 3
nanda-probe examples/triad-packet.negative-shortcut-lanes.json --input-format json --top-k 3
nanda-probe --suite examples/probe-corpus.json --input-format json --top-k 3
nanda-hgate examples/triad-packet.hgate-size-only.json --input-format json
nanda-search examples/triad-packet.source-weighting.json --input-format json --top-k 3
nanda-search examples/triad-packet.auto-query-memory.json --input-format json --query "lower operator debt route" --top-k 3
nanda-search examples/triad-packet.route-balanced-focus.json --input-format json --query "lower operator debt route" --route-cap 3 --route-triad-cap 1 --top-k 3
nanda-search examples/triad-packet.polarization-role-swap.json --input-format json --top-k 3
nanda-search examples/triad-packet.polarization-reversed-stop.json --input-format json --top-k 3
nanda-dogfood . --format json
```

## Current Status

V0 verdicts are intentionally conservative:

```text
PASS  - candidate triads match stable source bindings
WATCH - missing/incomplete/too-large/uncertain
VETO  - role swap, route contradiction, or weak candidate binding
```

Do not claim broad NANDA superiority yet. The current checker is the first
`SPARSE-TRIAD-0` implementation surface. It includes a 150-case synthetic
benchmark for clean payment bindings, swapped payment bindings, and route-splice
cases where individual facts are true but the combined route is wrong. The
route-splice benchmark also records exact-baseline false accepts.

Current benchmark:

```text
clean_pass:                         50/50
swap_veto:                          50/50
splice_veto:                        50/50
splice_exact_baseline_false_accept: 50/50
```

Current WAW corpus:

```text
waw_score:          3/3
structural_wins:    3/3
lexical_traps:      3/3
explainable_drifts: 3/3
```

Current dataset doctor fixture:

```text
verdict: WATCH
warnings: large_unbalanced_corpus, route_imbalance, hub_dominance, duplicate_current, weak_text_query
```

Current negative-lane fixture:

```text
without negative lane: customs
with negative lane:    certification
suppressed_peak:       customs
suppress_peak:         customs
group-aware suppress:  customs-shortcut
```

Current positive-lane fixture:

```text
accepted route:        certification
constructive boost:    applied
reinforced peak:       certification
reinforced group:      certification-route
```

Current resonant field fixture:

```text
route trap search:      WAW_RESONANCE
lexical trap:           customs
structural peak:        certification
phase:                  PHASE_LOCKED
standing wave:          STANDING_STABLE
energy:                 ENERGY_CONTAINED
proof state:            WATCH if packed peak remains thin
```

Current heavy 16k finance fixture:

```bash
scripts/generate-finance-16k-fixture.js .nanda/finance-16k-risk-cluster.json
scripts/test-heavy-16k.sh
```

Expected shape:

```text
triads:                 16,384
dataset-doctor:         PASS
pack6m full corpus:     FOCUS_REQUIRED
search full corpus:     FIELD_CONTESTED / WATCH
resonance:              NO_WAW_RESONANCE
focused proof:          9,620 triads, PACKED_RUNTIME_READY, WATCH
top_peak:               ai-demand
```

The generated JSON is intentionally not committed because it is about 6 MB.
Use this as an explicit load test when changing focus, proof, packed runtime
contracts, or finance-style route clustering.

## Roadmap

See [GOAL.md](GOAL.md), [ARCHITECTURE.md](ARCHITECTURE.md), and
[PLAN.md](PLAN.md).

The first useful proof is not a pitch. It is a benchmark:

```text
similar tokens, plausible facts, wrong binding -> NANDA returns VETO
lexical route trap -> NANDA interference peak selects the connected route
```

If simple symbolic rules or ordinary graph checks win, this project should say
that honestly.
