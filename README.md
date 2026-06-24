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

For the current command map, see [`COMMANDS.md`](COMMANDS.md). The runtime skill
file (`nanda-structural-gate/SKILL.md`) is the agent-facing command source, and
`COMMANDS.md` is the public human-facing index.

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

## Universality Rule

NANDA Structural Gate is universal infrastructure. Do not hardcode
project-specific names, routes, package names, file stems, product names, or
local conventions into the core. Domain specifics belong in route atlas data,
schemas, contracts, fixtures, or user-provided packets. Examples may mention a
real project, but the implementation must work on unrelated repositories with
different names.

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

## Command Map

The canonical public command list lives in [`COMMANDS.md`](COMMANDS.md). Use the
commands below as a quick smoke path after local install.

```bash
nanda-doctor
nanda-self-check
nanda-field-audit --format json
nanda-llmwave-big readiness-ladder --format json
nanda-llmwave-big claim-gate --claim field-core-sole-engine --format json
nanda-llmwave-big claim-gate --claim small-domain-llmwave --format json
nanda-llmwave-big claim-gate --claim nonlinear-memory --format json
nanda-llmwave-big claim-gate --claim llm-ready --format json
nanda-llmwave-big demo-domain --format json
```

Current intended claim boundary:

```text
field_core_as_sole_engine = true
small_domain_llmwave      = CLAIM_ALLOWED_LOCAL_ONLY
nonlinear_memory          = CLAIM_BLOCKED
llm_ready                 = CLAIM_BLOCKED
```

For the latest LLMWave-Big eval path, use:

```bash
nanda-llmwave-big demo-domain --format json

nanda-llmwave-big nonlinear-memory-ladder \
  --max-facts 100000 \
  --format json

nanda-llmwave-big schema-residual-engine \
  --format json

nanda-llmwave-big memory-physics \
  --format json

nanda-llmwave-big memory-proof-path \
  --format json

nanda-llmwave-big memory-final-proof \
  --format json

nanda-llmwave-big memory-final-proof \
  --profile rust \
  --format json

nanda-llmwave-big rust-corpus-build \
  --repo . \
  --out .nanda/llmwave-big-training/rust-corpus-artifact.json \
  --format json

nanda-llmwave-big rust-heldout-build \
  --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json \
  --out .nanda/llmwave-big-training/rust-heldout-suite.json \
  --format json

nanda-llmwave-big rust-focus-build \
  --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json \
  --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json \
  --out .nanda/llmwave-big-training/rust-focus-packet.json \
  --format json

nanda-llmwave-big rust-compile-evidence-build \
  --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json \
  --check-evidence .nanda/llmwave-big-training/cargo-check.json \
  --test-evidence .nanda/llmwave-big-training/cargo-test.json \
  --clippy-evidence .nanda/llmwave-big-training/cargo-clippy.json \
  --out .nanda/llmwave-big-training/rust-compile-evidence.json \
  --format json

nanda-llmwave-big rust-heldout-eval \
  --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json \
  --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json \
  --out .nanda/llmwave-big-training/rust-heldout-eval.json \
  --format json

nanda-llmwave-big strict-density-claim-gate \
  --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json \
  --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json \
  --heldout-eval .nanda/llmwave-big-training/rust-heldout-eval.json \
  --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json \
  --out .nanda/llmwave-big-training/strict-density.json \
  --format json

nanda-llmwave-big profile-density-build \
  --profile business \
  --corpus examples/llmwave-big-nonlinear-memory-corpus.json \
  --out .nanda/llmwave-big-training/business-density.json \
  --format json

nanda-llmwave-big profile-density-build \
  --profile contracts \
  --corpus examples/llmwave-big-contract-density-corpus.json \
  --out .nanda/llmwave-big-training/contracts-density.json \
  --format json

nanda-llmwave-big profile-density-build \
  --profile adversarial \
  --corpus examples/llmwave-big-adversarial-density-corpus.json \
  --out .nanda/llmwave-big-training/adversarial-density.json \
  --format json

nanda-llmwave-big multi-profile-density-suite \
  --rust-density .nanda/llmwave-big-training/strict-density.json \
  --profile-evidence adversarial=.nanda/llmwave-big-training/adversarial-density.json \
  --profile-evidence contracts=.nanda/llmwave-big-training/contracts-density.json \
  --profile-evidence business=.nanda/llmwave-big-training/business-density.json \
  --out .nanda/llmwave-big-training/multi-profile-density.json \
  --format json

nanda-llmwave-big density-proof-doctor \
  --suite .nanda/llmwave-big-training/multi-profile-density.json \
  --out .nanda/llmwave-big-training/density-proof-doctor.json \
  --format json

nanda-llmwave-big density-proof-doctor \
  --suite .nanda/llmwave-big-training/multi-profile-density.json \
  --min-fact-count 10 \
  --out .nanda/llmwave-big-training/density-proof-doctor-medium.json \
  --format json

nanda-llmwave-big density-ablation \
  --suite .nanda/llmwave-big-training/multi-profile-density.json \
  --format json

nanda-llmwave-big memory-final-proof \
  --profile rust \
  --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json \
  --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json \
  --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json \
  --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json \
  --heldout-eval .nanda/llmwave-big-training/rust-heldout-eval.json \
  --strict-density-evidence .nanda/llmwave-big-training/strict-density.json \
  --multi-profile-density-evidence .nanda/llmwave-big-training/multi-profile-density.json \
  --density-doctor-evidence .nanda/llmwave-big-training/density-proof-doctor.json \
  --format json

nanda-llmwave-big nonlinear-memory-eval \
  --corpus examples/llmwave-big-nonlinear-memory-corpus.json \
  --proof-policy scale-amortized \
  --format json

nanda-llmwave-big domain-eval \
  --artifact .nanda/llmwave-big-training/project-artifact.json \
  --ask-suite examples/llmwave-big-ask-eval.json \
  --hot-pack .nanda/llmwave-big-training/project.hot.bin \
  --chat-script .nanda/llmwave-big-training/chat.script \
  --chat-memory .nanda/llmwave-big-training/domain-chat-memory.json \
  --nonlinear-corpus examples/llmwave-big-nonlinear-memory-corpus.json \
  --format json
```

`nonlinear-memory-ladder` is the Phase 1 density instrument. It reports where
fixed-basis residual memory starts to beat a linear fact baseline, where the
basis overhead is repaid, and where collision pressure would need later gates.
It does not by itself prove nonlinear memory or broad LLM readiness.

`schema-residual-engine` is the Phase 2-3 controlled write path. It groups
observed facts into reusable schema keys, writes promoted routes as centroid
updates plus residual records, and keeps unsupported one-off facts as full
fallbacks.

`memory-physics` is the Phase 4-5 collision/noise path. It runs clean,
collision, and noise trials through shortcut-specific 32-byte anti-wave records
and reports whether false positives fall after suppression.

`memory-proof-path` is the Phase 6-8 bridge: held-out inference over schema
keys, basis economics from the density ladder, and route-balanced Wave Atlas
partitions. It remains controlled evidence, not final nonlinear-memory proof.

`memory-final-proof` is the Phase 9-12 gate. It checks field recall, the
LLMWave bridge, big-corpus evidence, and the final proof boundary. Until a real
big-corpus artifact and held-out suite are present, the expected honest verdict
is `FINAL_PROOF_GATE_BLOCKED_BY_BIG_CORPUS`.

Use `memory-final-proof --profile rust` for the first code-oriented proof
target. It changes the proof surface to Rust module owners, public API exports,
CLI dispatch, report printers, tests, compile evidence, and Rust-specific
forbidden shortcuts. It still keeps broad nonlinear-memory and LLM claims
blocked until a real Rust code corpus and held-out suite are present.

`rust-corpus-build` creates that first Rust structural corpus artifact by
scanning `.rs` files for modules, public exports, functions, CLI dispatch hints,
report-printer hints, and test evidence. It is a corpus input layer; it does
not by itself prove nonlinear memory.

`rust-heldout-build` consumes the Rust structural corpus artifact and builds
withheld route questions around module owners, public exports, CLI dispatch,
report printers, and test evidence. It also adds negative shortcuts such as
"compiled command implies LLM readiness". It is the next proof-prep layer, not
a final nonlinear-memory proof.

`rust-focus-build` consumes the corpus artifact and held-out suite, removes
the exact withheld facts from the focus window, and caps dominant routes so the
field is route-balanced. Once the three artifacts are passed into
`memory-final-proof --profile rust`, the expected honest blocker moves from
"missing corpus/focus" to `compile_test_evidence_bridge_missing`.

`rust-compile-evidence-build` consumes saved command-evidence JSON for
`cargo check`, tests, and clippy and links those pass/fail facts to the focus
packet. It intentionally does not run cargo as a hidden side effect. Once that
artifact is passed into final proof, the expected blocker becomes
`rust_heldout_inference_eval_missing`; nonlinear-memory and LLM claims remain
false.

`rust-heldout-eval` consumes `rust-focus-packet.json` and
`rust-heldout-suite.json`, then runs actual deterministic inference over
withheld route facts. The exact withheld facts are absent from the focus
window, so the eval must recover answers through local route/path
neighborhoods and reject false shortcuts such as "compiled command implies LLM
readiness". With compile evidence and held-out eval both passed into
`memory-final-proof --profile rust`, the honest next verdict is
`FINAL_PROOF_GATE_PROFILE_EVAL_READY_NOT_NONLINEAR_PROOF`: profile evidence is
ready, but broad nonlinear-memory and LLM claims remain false.

`strict-density-claim-gate` is the next Rust profile claim gate. It consumes
the corpus artifact, focus packet, held-out eval, and compile evidence, then
compares packed profile memory against a linear fact baseline. It requires
schema reuse, residual saving, packed bytes beating linear bytes, route
balance, held-out pass rate, false-shortcut rejection, and bounded collision
pressure. A pass gives `STRICT_DENSITY_PROFILE_PROVEN`, not a global claim.
When passed into `memory-final-proof --profile rust`, the honest next verdict
is `FINAL_PROOF_GATE_RUST_DENSITY_PROFILE_READY_NOT_GENERAL_LLM`; broad
nonlinear-memory and LLM claims remain false until multi-profile broad evals
exist.

`profile-density-build` is the generic non-Rust profile path. It adapts an
external relation corpus into the same density-evidence schema used by
`multi-profile-density-suite`, with local gates for amortized density win,
held-out quality, false-shortcut rejection, noise rejection, and bounded
collision pressure. A passing profile is still only profile evidence:
`PROFILE_DENSITY_PROVEN_NOT_GENERAL_LLM` does not prove broad nonlinear memory
or LLM readiness.

`multi-profile-density-suite` is the general nonlinear-memory guard. It accepts
one Rust density artifact plus additional independent profile artifacts such as
contracts, business-flow, natural-text, or noisy/adversarial profiles. A single
Rust artifact must remain blocked as
`MULTI_PROFILE_DENSITY_BLOCKED_BY_SINGLE_PROFILE`. Only multiple independent
passing profiles can produce
`MULTI_PROFILE_NONLINEAR_MEMORY_PROVEN_NOT_LLM`. Passing that suite into
`memory-final-proof --profile rust` is not enough by itself; final proof also
requires a `density-proof-doctor` artifact with medium-or-better evidence.
`llm_ready` remains false until broad LLM/chat evals exist.
Independence is checked by source signature: generic profile artifacts expose
`source.corpus_hash`, while legacy/strict artifacts fall back to a raw artifact
hash. Duplicate source hashes or identical artifacts block the suite with
`duplicate_or_missing_independent_profile_sources`.
Use `examples/llmwave-big-adversarial-density-corpus.json` as the built-in
route-collision, namespace, near-root, and shortcut-trap stress profile.
Run `density-proof-doctor` over the suite before treating the evidence as
strong. It reports `DENSITY_PROOF_BLOCKED`, `DENSITY_PROOF_WEAK`,
`DENSITY_PROOF_MEDIUM`, or `DENSITY_PROOF_STRONG`; small fixture corpora should
remain WEAK even when the formal suite gates pass. Pass the doctor artifact to
`memory-final-proof` with `--density-doctor-evidence`; WEAK/BLOCKED doctor
evidence keeps the final nonlinear-memory claim closed.
For fixture-scale development, `--min-fact-count 10` can demonstrate a
`DENSITY_PROOF_MEDIUM` path over the business/contracts/adversarial suite. This
is local medium evidence only: it may unlock
`FINAL_PROOF_GATE_NONLINEAR_MEMORY_READY_NOT_LLM`, never LLM readiness.
Run `density-ablation` when you need to know whether a suite depends on one
critical profile and whether the exposed density metrics beat the linear
baseline. It is a suite-level hook, not a proof by itself. The JSON output is
still wrapped in the unified field projection for agent-side comparison.

For nonlinear memory, inspect `corpus_driven_memory` before reading the broader
claim fields. That section is the actual fixture-driven density check: it
compares linear full-fact bytes with standalone fixed-basis bytes and amortized
fixed-basis residual bytes, then binds the result to held-out, negative, and
noise controls. A small corpus can honestly pass
`CORPUS_DRIVEN_AMORTIZED_DENSITY_OBSERVED` while still keeping
`nonlinear_memory_proven=false` because the standalone basis overhead has not
been repaid.

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
nanda-contract-gate --template --profile protocol --format json
nanda-contract-gate --input examples/contract-gate.protocol-pass.json --profile edo --format json
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
scripts/fetch-llmwave-big-gutenberg.sh
nanda-llmwave-big train README.md CHANGELOG.md LLMWAVE_BIG_ROADMAP.md src examples .nanda/external-corpus/gutenberg --out .nanda/llmwave-big-training/project-gutenberg-artifact.json --vocab-cap 65536 --transition-cap 262144 --active-chunk-cap 32768 --chunk-tokens 64 --format json
nanda-llmwave-big contract --format json
nanda-llmwave-big atlas --format json
nanda-llmwave-big active-core --format json
nanda-llmwave-big l2 --format json
nanda-llmwave-big hrr --format json
nanda-llmwave-big schema-bind --format json
nanda-llmwave-big l2-l3-couple --format json
nanda-llmwave-big decode-loop --format json
nanda-llmwave-big multi-schema --format json
nanda-llmwave-big schema-grow --format json
nanda-llmwave-big surface-generate --format json
nanda-llmwave-big reason-field --format json
nanda-llmwave-big dialogue-state --format json
nanda-llmwave-big mini-chat-eval --format json
nanda-llmwave-big query-wave --text "Has customs cleared the goods?" --format json
nanda-llmwave-big multi-peak-field --text "Has customs cleared the goods?" --format json
nanda-llmwave-big lens-scan --text "Has customs cleared the goods?" --format json
nanda-llmwave-big mature-anti-wave --text "Has customs cleared the goods?" --format json
nanda-llmwave-big evidence-proof --text "Has customs cleared the goods?" --evidence-mode release-confirmed --format json
nanda-llmwave-big answer-surface --text "Has customs cleared the goods?" --evidence-mode release-confirmed --format json
nanda-llmwave-big field-feedback --text "Has customs cleared the goods?" --evidence-mode release-confirmed --decision accept --format json
nanda-llmwave-big feedback-memory --text "Has customs cleared the goods?" --evidence-mode release-confirmed --decision accept --format json
nanda-llmwave-big feedback-aware-field --text "Has customs cleared the goods?" --memory-mode accept --format json
nanda-llmwave-big applied-anti-memory --format json
nanda-llmwave-big memory-store --path .nanda/llmwave-big-memory.json --action apply --decision accept --format json
nanda-llmwave-big learning-eval --format json
nanda-llmwave-big memory-consolidate --format json
nanda-llmwave-big run --evidence-mode release-confirmed --decision accept --format json
nanda-llmwave-big core-eval --format json
nanda-llmwave-big word-birth --format json
nanda-llmwave-big surface-production --format json
nanda-llmwave-big surface-reconstruct --format json
nanda-llmwave-big surface-corpus-eval --format json
nanda-llmwave-big surface-bank-build --format json
nanda-llmwave-big surface-bank-validate --format json
nanda-llmwave-big surface-bank-fixture --corpus examples/llmwave-big-surface-corpus.json --format json
nanda-llmwave-big surface-bank-fixture --corpus examples/llmwave-big-surface-corpus-ru.json --format json
nanda-llmwave-big surface-raw-induce --corpus examples/llmwave-big-raw-surface-corpus-ru.json --format json
nanda-llmwave-big surface-raw-induce --corpus examples/llmwave-big-raw-surface-corpus-ru-noisy.json --format json
nanda-llmwave-big surface-raw-induce --corpus examples/llmwave-big-raw-surface-corpus-ru-derived.json --format json
nanda-llmwave-big demo-domain --format json
nanda-llmwave-big train README.md CHANGELOG.md LLMWAVE_BIG_ROADMAP.md src examples --out .nanda/llmwave-big-training/project-artifact.json --format json
nanda-llmwave-big ask --artifact .nanda/llmwave-big-training/project-artifact.json --text "what does declaration require" --top-k 5 --format json
nanda-llmwave-big ask-eval --artifact .nanda/llmwave-big-training/project-artifact.json --suite examples/llmwave-big-ask-eval.json --top-k 5 --format json
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
nanda-map-code .
nanda-boundary-economics . --format json
nanda-boundary-economics . --atlas .nanda/route-atlas.json --route ime-display-flow --owner LayIbusEngine --format json
nanda-build-atlas . --out .nanda/route-atlas.json
nanda-guard-action .nanda/route-atlas.json --symptom "IME not visible" --action-id ime.activate_engine --boundary-economics
nanda-guard-diff .nanda/route-atlas.json --action-id ime.show_candidate --diff git.diff --boundary-economics
nanda-profile-guards . --iterations 50 --format json
nanda-release-gate .nanda/route-atlas.json
nanda-dogfood . --refactor-plan --boundary-economics --format json
nanda-self-check
```

`nanda-contract-gate` is the universal document-flow layer for contracts,
appendices, protocol-of-disagreements, EDI/EDO, and similar role-sensitive
checks. It does not hardcode a project or counterparty: the packet supplies
parties, protocol author, source/proposed clauses, risk tags, and optional EDI
messages. Treat `STRUCTURAL_PASS_NOT_LEGAL_APPROVAL` as structural coherence
only; signing still requires legal/accounting review.

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
`examples/self-dogfood.nanda.json` packet when present. If no curated packet
exists, it builds a low-confidence repo auto-field. Auto-field is review-only
until the agent provides a precise `action_id`, evidence, and route-specific
verification.
`nanda-map-code` is the refactor planning pass for Rust files or repositories.
On a file, it clusters functions, reports cross-cluster dependencies, suggests
target files, and marks extraction risk. On a directory, it returns a
repo-level `repo-code-map`. Use `nanda-dogfood . --refactor-plan` when you want
the normal structural verdict plus repository-level code-boundary
recommendations in one packet. The dogfood refactor plan scans multiple Rust
files and reports `risk_files`; it should not treat `src/main.rs` as the whole
repository.
`nanda-boundary-economics` is the refactor boundary layer. It does not suggest
split/merge from file size. It applies `NO EVIDENCE => NO CUT` and returns a
JSON `boundary_decision` with verdict, score components, evidence, allowed
files, forbidden routes, required tests, and repair contract. Use repo-wide
mode to find possible split pressure; use route-scoped mode with `--atlas`,
`--route`, and `--owner` before a concrete refactor. Route-scoped mode starts
from `atlas.routes[route].allowed_files` and then narrows by owner/path so the
contract does not drag unrelated routes into the evidence. If an explicit
`--owner` does not match the selected route atlas, the verdict is `WATCH` and
`safe_to_edit=false`; the command must not fall back to the whole route.
Guard commands expose the same contract under `boundary_economics` while
keeping top-level `boundary_decision` for compatibility. Verdicts are
`SPLIT_STRONG`, `SPLIT_WEAK`, `KEEP`, `MERGE_CANDIDATE`, `VETO`, and `WATCH`.
`WATCH` means do not cut; `VETO` means stop; `KEEP` means do not touch the
boundary; `SPLIT_STRONG` allows refactor only inside the returned contract;
`SPLIT_WEAK` is a small preparatory step plus human review; `MERGE_CANDIDATE`
needs a separate merge plan or review.
`nanda-build-atlas` writes reusable route memory to `.nanda/route-atlas.json`.
`nanda-guard-action` is the fast pre-edit check: symptom plus `action_id` must
resolve to an atlas route. `nanda-guard-diff` is the post-edit check: changed
files must stay inside the selected route capsule. Empty, unreadable, or
unparseable diffs return `WATCH` with `safe_to_edit=false`, not PASS. Diff
files from a different git repository return `WATCH` with
`reason=diff_source_repo_mismatch`. Intentional cross-route edits must use an
explicit shared contract action such as `shared.manual_toggle_contract`,
`shared.text_edit_contract`, `shared.candidate_contract`, or
`shared.layout_sync_contract`; otherwise route crossing is `VETO` and the
report names changed routes, shared candidates, and suggested shared actions.
Use `shared.version_bump_contract` only for release metadata diffs. It is scoped
to `Cargo.toml`, `Cargo.lock`, `VERSIONING.md`, extension metadata/version JS,
and explicitly versioned README/HOW_IT_WORKS edits. The guard checks Cargo,
lockfile, extension `version-name`, numeric metadata version, JS `APP_VERSION`,
and stale version tokens before returning PASS.
`nanda-release-gate` is a checklist summary over the atlas before publishing.
`nanda-profile-guards` measures the atlas-first workflow before optimization:
build-atlas, guard-action, guard-diff, map-code, dogfood, and the warm
`nanda-serve` guard path. Cold `guard-action`/`guard-diff` includes process
startup and atlas JSON load; `serve_guard_*` keeps one process alive and caches
the atlas. Use it before making performance claims or moving full-field checks
into the protected edit loop.
`nanda-report` is agent-first: it returns a JSON decision packet by default.
Use `--format md` only when a human-facing report is explicitly needed.
`nanda-map` exposes the core structural map: source/candidate group sizes,
interference matrix, dominant source group, mixed candidate groups, route
field, owner gravity, negative route hits, structural energy, and repair tasks.
Use `route_field` to see route/layer/owner/entrypoint/output/evidence/scope
coordinates. Use `owner_gravity.conflicts` to catch two owners pulling one
decision. Use `negative_routes.hits` to catch anti-modes such as adapter/UI
decision ownership, test-only paths leaking into runtime, helpers owning
decisions, or experiments affecting stable routes.
`codex_failure_field` is an opt-in edit firewall. Add
`failure_contract.enabled=true` with a user symptom, evidence, selected
`action_id`, allowed/forbidden routes, runtime snapshot, namespace terms, and
route-specific verification. It returns `PASS`, `ANALYSIS_INSUFFICIENT`,
`VETO`, or `HARD_STOP`. Use it to catch symptom/action mismatch, scope creep,
namespace confusion, runtime blindness, fake verification, unproven
hypotheses, example-specific patches, and missing checkpoints before editing.
When enabled, failure-field repairs are placed first in top-level
`repair_queue`, before generic structural coherence repairs. For example,
runtime evidence plus a code-edit action should first say to repair the runtime
route and not edit candidate generation.
`nanda-hgate` is the hierarchical gate for large packets. It runs one
global map/check, splits by linked group, runs local gates, and returns
`STRUCTURALLY_ACCEPTED` only when the global `WATCH` is size-only and every
local branch is `PASS`. If `foreign_pull`, owner conflicts, negative route
hits, conflicts, or any local `VETO` exist, it returns `REPAIR_REQUIRED`.
`nanda-extract` converts simple arrow text into a triad packet. The supported
line format is `subject -> relation -> object [route=x group=y ...]`, with
`## triads` and `## candidate_triads` sections.
If a repository has no curated `examples/self-dogfood.nanda.json`,
`nanda-dogfood <repo>` builds a low-confidence auto route-field from source,
config, script, UI/status, runtime, and test files. That fallback is deliberately
review-only: it should help the agent see contours, not grant edit permission.
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
`nanda-llmwave-big l2` started as the v181-v190 L2 Word Field contract and now
includes the v361-v390 L2 Wave Field Runtime: typed prefix wave, candidate
surface wave, family resonance, suffix/program resonance, L3 phase bias, and
local anti-wave suppression. In the built-in sample, prefix `счет` ranks
`счете` as the top surface while suppressing the near-root prefix trap
`счетчик`. This is `L2_WAVE_RUNTIME_READY_NOT_CHAT`: L2 surface ranking works,
but chat readiness and nonlinear memory proof remain false.
`nanda-llmwave-big hrr` adds the v391-v430 HRR/VSA binding core. It binds
role/filler waves such as `supplier ⊗ Honglu`, superposes them into a schema
field, unbinds by role, and passes the recovered cue through cleanup memory.
The first stable implementation uses bipolar VSA elementwise binding; naive
circular convolution remains a planned comparison because it was not stable
enough in the initial fixture. The report must keep
`nonlinear_memory_proven=false` and `llm_ready=false`.
`nanda-llmwave-big schema-bind` adds the v431-v455 L3 Schema Binding core. It
connects schema record `101` (`supplier issues invoice`) to role/filler wave
bindings, recovers `subject:supplier -> Honglu` and
`object:document -> invoice`, and rejects the role-swap trap
`invoice issues Honglu`. This is still fixture-level schema cognition, not LLM
readiness or nonlinear memory proof.
`nanda-llmwave-big l2-l3-couple` adds the v456-v480 L2/L3 coupling core. It
lets an L2 surface probe propose candidates, then applies the L3 schema role as
a phase-bias/rerank layer. In the sample, raw L2 prefers `inventory` for prefix
`in`, but the L3 `object:document` role lifts `invoice`; the disagreement trap
rejects `invoice` when the active L3 slot expects `subject:supplier -> Honglu`.
This is a local L2/L3 feedback loop, not chat readiness or nonlinear proof.
`nanda-llmwave-big decode-loop` adds the v481-v520 recurrent L2/L3 loop. It
walks a tiny role cursor `subject:supplier -> operator -> object:document`,
accepts `Honglu issues invoice`, updates L2 context energy and L3 schema phase
after each accepted step, and stops the bad continuation `invoice issues
Honglu`. This is the first tiny schema-shaped generation loop, still not broad
chat or nonlinear-memory proof.
`nanda-llmwave-big multi-schema` adds the v521-v560 competition layer. It keeps
four active schemas in the field (`supplier issues invoice`, `buyer pays
invoice`, `customs checks declaration`, `lab issues protocol`), selects the
coherent `supplier-docs` route for `Honglu issues invoice`, and rejects the
route splice `Honglu pays invoice`: every piece exists somewhere, but no single
schema owns the whole route. This is fixture-level route competition, not broad
reasoning.
`nanda-llmwave-big schema-grow` adds the v561-v620 schema-memory growth layer.
It scans a small embedded observation corpus, promotes repeated route forms into
fixed `LearnedSchema32` records, and rejects the one-off `warehouse signs
invoice` trap. This is the first growth step for L3 memory, but the corpus is
still embedded and the report keeps chat readiness and nonlinear proof false.
`nanda-llmwave-big surface-generate` adds the v621-v700 open-surface generation
layer. It takes the learned `supplier-docs` schema and materializes `Honglu
issued invoice PI-03 to Rustrade` through fixed `SurfaceStep32` records,
combining surface programs, grammar atoms, and evidence-copy spans. It rejects
the route-splice surface `Honglu paid invoice PI-03 to Rustrade`; this is still
a constrained generation fixture, not free-form chat.
`nanda-llmwave-big reason-field` adds the v701-v780 multi-step reasoning field.
It propagates the generated invoice premise through three fixed
`ReasoningHop32` records: invoice issuance creates a payment expectation,
payment/declaration context feeds customs checking, and customs checking still
requires declaration evidence. It rejects the shortcut `customs cleared goods`.
`nanda-llmwave-big dialogue-state` adds the v781-v860 dialogue state layer. It
answers the question `Has customs cleared the goods?` with a constrained
`Not proven` response, keeps the declaration-evidence boundary, and rejects the
unsupported answer `Yes, customs cleared the goods.` This is single-turn state
control, not multi-turn chat readiness.
`nanda-llmwave-big mini-chat-eval` adds the v861-v950 eval boundary over the
current schema-growth, surface-generation, reasoning, and dialogue chain. It
checks five embedded cases: grounded not-proven answer, unsupported-certainty
rejection, route-splice rejection, one-off schema-noise rejection, and exact
constrained surface generation. Its passing verdict is a mini chat candidate
for this fixture chain only, not a general LLM, not broad chat readiness, and
not nonlinear-memory proof.
`nanda-llmwave-big query-wave` starts the Mature Field Core path in v951-v1000.
It converts input text into a compact query wave with L2 surface excitation,
L3 role/operator hints, question/assertion polarity, and a fixed
`QueryWaveRecord32`. Its eval checks that English and Russian paraphrases of a
customs-clearance status question hit the same route while an assertion trap is
not treated as a safe question. This is text-to-field excitation, not mature
field selection and not chat readiness.
`nanda-llmwave-big multi-peak-field` adds v1001-v1060. It takes the query wave
and excites competing schema peaks, computes energy/margin/leakage, and
classifies the field as `STABLE_PEAK`, `CONTESTED`, `NO_ANSWER`, or
`REJECTED`. It deliberately keeps `safe_to_answer=false`: answer permission is
a later lens decision, not a property of a raw peak.
`nanda-llmwave-big lens-scan` adds v1061-v1140. It scans the same raw field
through role, evidence, temporal, causal, contradiction, surface, and answer
lenses. A stable customs-clearance peak is still blocked when the evidence and
answer lenses do not permit a claim. This is the first mature-field lens layer,
not chat readiness.
`nanda-llmwave-big mature-anti-wave` adds v1141-v1210. It compiles WATCH/BLOCK
lenses into fixed `AntiLaneRecord32` lanes, suppresses the unsupported answer
permission, and preserves the route peak for later evidence proof. This is local
anti-wave field shaping, not answer permission and not chat readiness.
`nanda-llmwave-big evidence-proof` adds v1211-v1280. It binds a stable route
peak to a compact evidence proof record. Missing evidence keeps the answer
blocked, while `--evidence-mode release-confirmed` grants only local answer
permission for the fixture route. This is evidence-bound permission, not general
chat readiness or nonlinear-memory proof.
`nanda-llmwave-big answer-surface` adds v1281-v1350. It materializes the proof
state through fixed answer templates: missing evidence becomes a `Not proven`
surface, and bound evidence becomes a local confirmation that copies the evidence
ref. This is constrained answer text, not free-form generation.
`nanda-llmwave-big field-feedback` adds v1351-v1420. It converts accept/reject
decisions over constrained answer surfaces into fixed `FieldFeedbackRecord32`
records: accept reinforces the evidence-bound route, reject writes local
anti-memory. This is local feedback, not persistent training.
v1421-v1900 closes the first applied field-core loop. `feedback-memory` turns
feedback into fixed `AppliedMemoryRecord32` packets, `feedback-aware-field`
applies those packets to the next field pass, `applied-anti-memory` proves the
reject lane suppresses the false route while preserving the true route,
`memory-store` writes a reusable JSON memory packet, `learning-eval` measures
before/after route lift and suppression, `memory-consolidate` controls duplicate
growth, `run` executes the full fixture pipeline, and `core-eval` reports
`CORE_RUNTIME_READY_FIXTURE`. This is now an applied fixture runtime core, still
not a full LLM and still not nonlinear-memory proof.
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

`nanda-llmwave-big surface-bank-build` adds the v281-v290 observed surface-bank
builder. It takes a small embedded business-form corpus, promotes observed
suffix families into `SurfaceFamily32` records, emits `SurfaceBinding8` virtual
forms, rejects non-family fragments into copy/provisional paths, and tests
held-out reconstructions such as `invoicing`, `customing`, and `routing`. Its
state remains `OBSERVED_BANK_BUILD_PASS_NOT_DENSITY_PROOF`: this is bank
construction evidence, not real broad corpus training.

`nanda-llmwave-big surface-bank-validate` adds the v291-v300 validator around
that bank. It checks positive held-out forms, negative controls such as
`invoiceing` and rare identifier family traps, and order-shuffle stability. Its
state remains `VALIDATION_PASS_NOT_GENERAL_PROOF`: this catches known false
families in the embedded suite, but does not prove real corpus training,
free-form spelling, or nonlinear surface memory.

`nanda-llmwave-big surface-bank-fixture` adds the v301-v310 external corpus
fixture loader. It reads `examples/llmwave-big-surface-corpus.json`, validates
surface families, held-out reconstructions, false-family controls, and rare
copy-span paths from JSON instead of Rust constants. Its state remains
`EXTERNAL_FIXTURE_PASS_NOT_GENERAL_PROOF`: fixture IO works, but real corpus
training and nonlinear surface memory are still false.
The companion Russian fixture,
`examples/llmwave-big-surface-corpus-ru.json`, checks the same path on Cyrillic
business forms such as `счет`, `договор`, `декларация`, `сертификат`,
`платеж`, and `маршрут`, with rare exact forms like `ТР ТС 021/2011`.

`nanda-llmwave-big surface-raw-induce` adds the v311-v320 raw-form induction
step. It reads `examples/llmwave-big-raw-surface-corpus-ru.json`, where the
input is a flat list of Russian word forms plus a suffix inventory; roots are
not provided to the inducer. The current fixture induces six Cyrillic families
and reconstructs held-out forms, but still reports
`RAW_INDUCTION_PASS_NOT_GENERAL_PROOF`.
The noisy companion fixture,
`examples/llmwave-big-raw-surface-corpus-ru-noisy.json`, adds near-root
collisions such as `счетчик`, `маршрутизатор`, and `сертификатор`. It reports
`NOISY_RAW_INDUCTION_PASS_NOT_GENERAL_PROOF` only when these collision roots
are rejected instead of promoted as families.
The derived companion fixture,
`examples/llmwave-big-raw-surface-corpus-ru-derived.json`, removes the manual
suffix inventory from the input. `surface-raw-induce` derives suffixes from
repeated observed form tails, selects non-overlapping candidate roots, and
still rejects near-root collisions. It reports
`DERIVED_SUFFIX_RAW_INDUCTION_PASS_NOT_GENERAL_PROOF`, not broad morphology or
nonlinear memory proof.

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
`nanda-llmwave-big train` starts the real-corpus training path after the
fixture-era core work. It recursively loads UTF-8 corpus files, deduplicates
files and chunks, builds compact token/transition/chunk/schema-hint records,
writes a Wave Atlas training artifact, and runs a held-out next-token resonance
check. The command keeps cold text and JSON outside the hot Active Core and
reports the estimated hot record budget. Its successful verdict is
`TRAINING_ARTIFACT_READY_NOT_LLM`: the corpus path is real, but broad chat,
nonlinear memory proof, and cache-only execution remain explicitly unproven.
Use `scripts/fetch-llmwave-big-gutenberg.sh` to fetch a reproducible
public-domain text slice into `.nanda/external-corpus/gutenberg`. Generated
corpora and training artifacts stay under `.nanda/` and are not committed.
`nanda-llmwave-big ask` reads that compiled training artifact back into the
field path. It builds a query wave, scores schema/chunk/transition peaks, and
answers only when the trained-artifact field is focused. Its ready state is
`ARTIFACT_FIELD_ANSWER_READY_NOT_GENERAL_LLM`: useful narrow answering from the
artifact, not a broad chat model.
`nanda-llmwave-big ask-eval` runs multiple artifact-grounded questions and
tracks answer accuracy, false positives, and false negatives. A chunk-only peak
is review evidence, not answer permission; safe answers require a focused
schema peak.
`nanda-llmwave-big pack-hot` converts a training artifact JSON into a compact
binary hot Active Core pack. The pack stores fixed-size numeric records only:
token hashes, transition hashes, chunk hashes, phases, counts, and schema
hints. It excludes cold strings and JSON from the hot pack and reports actual
file bytes against the hot budget. Its successful verdict is
`HOT_PACK_READY_NOT_CACHE_ONLY_PROOF`: compact binary storage is real, but
cache-only execution and broad chat readiness remain unproven.
`nanda-llmwave-big ask-hot` scans that binary hot pack for schema and
transition peaks. It may use the cold artifact only to decode labels for human
output, while reporting `json_used_in_hot_scan=false`. The hot scan applies a
role-polarity lens over `subject -> relation -> object`: aligned role order can
answer, `OBJECT_FOREIGN_PULL` stays review-only, and reversed subject/object
order is a hard stop. Its successful verdict is
`HOT_FIELD_ANSWER_READY_NOT_GENERAL_LLM`, still not cache-only execution proof
and still not broad chat readiness.
`nanda-llmwave-big learn-hot` ingests batch feedback JSON and writes a
persistent hot-memory overlay. `ask-hot --memory memory.json` applies those
learned records on the next field pass, so accepted corrections can become
learned schema peaks without hand-entering one fact at a time. Treat
`HOT_LEARNING_MEMORY_WRITTEN_NOT_GRADIENT_TRAINING` as real persistent feedback
learning for the hot retrieval layer, not transformer-style training.
`nanda-llmwave-big chat-hot` is the no-hand-JSON shell over `ask-hot` and
`learn-hot`. Use `ask <text>` for queries and
`learn accept: subject | relation | object` or
`learn reject: subject | relation | object` for corrections. In scripted mode
with `--script`, it is regression-testable; in stdin mode, it is the first
usable conversation loop over the hot memory.
`nanda-llmwave-big demo-domain` is the one-command smoke for that whole narrow
path. It writes a bundled tiny corpus, compiles it, packs the binary hot core,
runs scripted hot-chat eval, runs the small-domain eval, and reports
`DEMO_DOMAIN_PASS_NOT_BROAD_LLM` only when all local components pass. Treat it
as proof that the demo route works, not proof of broad LLM or general nonlinear
memory readiness.
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
nanda-dogfood . --build-atlas --atlas-out .nanda/route-atlas.json
nanda-build-atlas . --out .nanda/route-atlas.json
nanda-guard-action .nanda/route-atlas.json --symptom "IME not visible" --action-id ime.activate_engine
nanda-guard-diff .nanda/route-atlas.json --action-id ime.show_candidate --diff git.diff
nanda-guard-diff .nanda/route-atlas.json --action-id shared.version_bump_contract --diff version.diff
nanda-field-report --from search-result.json --format json
nanda-profile-guards . --iterations 50 --format json
nanda-release-gate .nanda/route-atlas.json
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
If `owner_gravity.conflicts` or `negative_routes.hits` are non-empty, treat the
map the same way: no PASS until the decision owner is singular and adapters,
UI, tests, helpers, and experiments are back behind their allowed boundaries.
`structural_energy.field_tension` is the compact numeric signal for how much
route pressure remains; `repair_queue` is the machine-readable list of minimum
repairs. Failure-field repairs are failure-first: a blocked action/evidence
mismatch should appear before lower-priority coherence repair.
If `codex_failure_field.verdict` is `HARD_STOP`, do no tools, no code, no
restart. If it is `VETO`, repair the action/evidence/route mismatch first. If
it is `ANALYSIS_INSUFFICIENT`, choose a more precise `action_id`, add evidence,
namespace ambiguous terms, or attach route-specific verification before
editing.

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

Use a tiered workflow, not full-field scanning on every prompt:

1. Always-on micro gate: STOP, edit permission, action_id, symptom/action fit.
2. Route snapshot gate: use route atlas and guard-action before code edits.
3. Full route-field gate: use dogfood/map/hgate for multi-route changes.
4. Release gate: run tests, dogfood, route-specific smoke, and release-gate.

Atlas-first, gate-on-diff workflow:

```bash
mkdir -p .nanda
nanda-build-atlas . --out .nanda/route-atlas.json
nanda-guard-action .nanda/route-atlas.json --symptom "IME not visible" --action-id ime.activate_engine --boundary-economics
nanda-guard-diff .nanda/route-atlas.json --action-id ime.show_candidate --diff git.diff --boundary-economics
nanda-boundary-economics . --format json
nanda-boundary-economics . --atlas .nanda/route-atlas.json --route ime-display-flow --owner LayIbusEngine --format json
nanda-profile-guards . --iterations 50 --format json
nanda-dogfood . --out-dir .nanda/
nanda-dogfood . --refactor-plan --boundary-economics --format json
nanda-release-gate .nanda/route-atlas.json
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
