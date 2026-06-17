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
        ├── nanda-doctor
        ├── nanda-dogfood
        ├── nanda-eval
        ├── nanda-extract
        ├── nanda-feedback
        ├── nanda-index
        ├── nanda-map
        ├── nanda-search
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
nanda-comb examples/triads.code-flow-splice.md --domain code --depth 2
mkdir -p .nanda
nanda-extract examples/route-trap.raw.txt --out .nanda/route-trap.json
nanda-index examples/triad-packet.interference-search-route-trap.json --input-format json --out .nanda/index.json
nanda-search .nanda/index.json --input-format json --query-file examples/triad-packet.interference-search-route-trap.json --query-format json --top-k 3
nanda-search examples/triad-packet.interference-search.json --input-format json --top-k 3
nanda-search examples/triad-packet.interference-search-noisy.json --input-format json --format text
nanda-search examples/triad-packet.interference-search-route-trap.json --input-format json --top-k 3
nanda-search examples/triad-packet.interference-search-route-trap.json --input-format json --top-k 3 > .nanda/search.json
nanda-feedback .nanda/search.json --decision accept --note "accepted focused peak"
nanda-eval --case examples/triad-packet.interference-search-route-trap.json:certification:FOCUSED --case examples/triad-packet.interference-search-noisy.json:certification:WATCH
nanda-doctor
nanda-dogfood .
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
`nanda-report` is agent-first: it returns a JSON decision packet by default.
Use `--format md` only when a human-facing report is explicitly needed.
`nanda-map` exposes the core structural map: source/candidate group sizes,
interference matrix, dominant source group, mixed candidate groups, and repair
tasks.
`nanda-extract` converts simple arrow text into a triad packet. The supported
line format is `subject -> relation -> object [route=x group=y ...]`, with
`## triads` and `## candidate_triads` sections.
`nanda-index` builds a reusable memory packet from one or more triad packets or
Markdown worksheets.
`nanda-search` is the v1.0 memory-index retrieval surface. It treats `triads`
as memory and either the same packet's `candidate_triads` or a separate
`--query-file` as the partial query, then returns top-k route/group peaks with
support, foreign pulls, missing edges, and an answer projection.
`nanda-feedback` is the feedback-memory surface. It records whether a search
peak was accepted, rejected, or kept under WATCH, together with margin, support
ids, anti ids, and a compact memory patch.
`nanda-eval` is the regression surface. It checks expected peak/state pairs
so interference changes are measured before they are trusted.
`nanda-doctor` is the v1.0 release smoke test. It runs built-in focused/noisy
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
core_version: sparse-triad-v1.0-release
wave_dim: 1024
```

`v1.0-release` keeps recursive topology combing, v0.5 structural peak search,
v0.6 reusable memory indexes, v0.7 arrow-text extraction, v0.8 feedback
packets, and v0.9 regression evaluation, then adds a self-contained doctor
check. The search path is intentionally small and universal: encode triads as
slot-bound waves, superpose a partial query, score memory routes/groups by
interference, then interpret, record, test, and smoke-check the top peaks.
If `foreign_pull` is non-empty, strict gate output is not `PASS`; repair the
named candidate triads or split the route first.

Interference search output:

```text
peak
score
peak_margin
lexical_baseline
wins_over_lexical_baseline
peak_decision.state
peak_decision.safe_to_answer
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
nanda-search .nanda/index.json --input-format json --query-file query.json --query-format json --top-k 5
nanda-feedback .nanda/search.json --decision watch --note "margin too low"
nanda-eval --case route-trap.json:certification:FOCUSED --case noisy.json:certification:WATCH
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
- run `nanda-map` first to inspect `mixed_candidate_groups` and `foreign_pull`;
- treat non-empty `foreign_pull` as a repair stop;
- do not force one global PASS when the graph exceeds size limits;
- use `linked-group` split to produce paired source/candidate worksheets;
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

Current release: `v1.0.1`.

Release notes are maintained in [CHANGELOG.md](CHANGELOG.md). Before tagging a
release, run:

```bash
scripts/test-local.sh
scripts/test-edge-cases.sh
scripts/benchmark-v0.sh
nanda-doctor
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

## Roadmap

See [GOAL.md](GOAL.md), [ARCHITECTURE.md](ARCHITECTURE.md), and
[PLAN.md](PLAN.md).

The first useful proof is not a pitch. It is a benchmark:

```text
similar tokens, plausible facts, wrong binding -> NANDA returns VETO
```

If simple symbolic rules or ordinary graph checks win, this project should say
that honestly.
