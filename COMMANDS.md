# NANDA Command Map

This file is the public command index. `nanda-structural-gate/SKILL.md` remains
the runtime instruction file for Codex agents. `UNIFIED_FIELD_REFACTOR_PLAN.md`
tracks what each claim means and which claims are still blocked.

## Health And Install

```bash
scripts/install-local.sh
nanda --version
nanda-doctor
nanda-self-check
```

Use these before release or after adding a new CLI subcommand:

```bash
cargo fmt --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
scripts/test-edge-cases.sh
scripts/test-local.sh
scripts/install-local.sh
nanda-self-check
```

## Structural Gate

```bash
nanda-init-md --task-id local-check --domain general --query "check routes"
nanda-gate-md nanda-task-local-check.md --task-id local-check --domain general
nanda-check --triads examples/triad-packet.route-splice.json --input-format json
nanda-map examples/triads.code-flow-splice.md --domain code --normalize-paths
nanda-split-md examples/triads.code-flow-splice.md --by linked-group --out-dir split/
nanda-split examples/triad-packet.route-splice.json --input-format json --by linked-group --out-dir split-json/
```

## Repo Guard

Build an atlas once, then guard actions and diffs against route confusion.

```bash
nanda-build-atlas . --out .nanda/route-atlas.json
nanda-guard-action .nanda/route-atlas.json \
  --symptom "IME not visible" \
  --action-id ime.activate_engine \
  --boundary-economics
nanda-guard-diff .nanda/route-atlas.json \
  --action-id ime.show_candidate \
  --diff git.diff \
  --boundary-economics
nanda-release-gate .nanda/route-atlas.json
```

## Boundary Economics

Use this before refactoring. `WATCH` means do not cut yet.

```bash
nanda-boundary-economics . --format json
nanda-boundary-economics . \
  --atlas .nanda/route-atlas.json \
  --route ime-display-flow \
  --owner LayIbusEngine \
  --format json
nanda-dogfood . --refactor-plan --boundary-economics --format json
```

## Field Core

```bash
nanda-field-report --from search-result.json --format json
nanda-field-audit --format json
nanda-field-equivalence \
  --structural-from search-result.json \
  --packed-from pack6m-result.json \
  --cognitive-from llmwave-big-result.json \
  --format json
nanda-field-cutover --suite structural-standard --format json
```

Current intended claim boundary:

```text
field_core_as_sole_engine = true
llm_ready = false
nonlinear_memory_proven = false
```

## LLMWave Readiness And Claims

```bash
nanda-llmwave-big readiness-ladder --format json
nanda-llmwave-big claim-gate --claim field-core-sole-engine --format json
nanda-llmwave-big claim-gate --claim small-domain-llmwave --format json
nanda-llmwave-big claim-gate --claim nonlinear-memory --format json
nanda-llmwave-big claim-gate --claim llm-ready --format json
nanda-llmwave-big demo-domain --format json
```

Expected boundary today:

```text
small-domain-llmwave      CLAIM_ALLOWED_LOCAL_ONLY
nonlinear-memory          CLAIM_BLOCKED
llm-ready                 CLAIM_BLOCKED
```

## Nonlinear Memory Eval

Strict mode keeps the general nonlinear-memory claim blocked until the whole
sweep beats the linear baseline.

```bash
nanda-llmwave-big nonlinear-memory-ladder --max-facts 100000 --format json
nanda-llmwave-big schema-residual-engine --format json
nanda-llmwave-big memory-physics --format json
nanda-llmwave-big memory-proof-path --format json
nanda-llmwave-big memory-final-proof --format json
nanda-llmwave-big memory-final-proof --profile rust --format json
nanda-llmwave-big rust-corpus-build --repo . --out .nanda/llmwave-big-training/rust-corpus-artifact.json --format json
nanda-llmwave-big rust-heldout-build --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --out .nanda/llmwave-big-training/rust-heldout-suite.json --format json
nanda-llmwave-big rust-focus-build --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --out .nanda/llmwave-big-training/rust-focus-packet.json --format json
nanda-llmwave-big rust-compile-evidence-build --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --check-evidence .nanda/llmwave-big-training/cargo-check.json --test-evidence .nanda/llmwave-big-training/cargo-test.json --clippy-evidence .nanda/llmwave-big-training/cargo-clippy.json --out .nanda/llmwave-big-training/rust-compile-evidence.json --format json
nanda-llmwave-big rust-heldout-eval --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --out .nanda/llmwave-big-training/rust-heldout-eval.json --format json
nanda-llmwave-big strict-density-claim-gate --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --heldout-eval .nanda/llmwave-big-training/rust-heldout-eval.json --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json --out .nanda/llmwave-big-training/strict-density.json --format json
nanda-llmwave-big profile-density-build --profile business --corpus examples/llmwave-big-nonlinear-memory-corpus.json --out .nanda/llmwave-big-training/business-density.json --format json
nanda-llmwave-big profile-density-build --profile contracts --corpus examples/llmwave-big-contract-density-corpus.json --out .nanda/llmwave-big-training/contracts-density.json --format json
nanda-llmwave-big profile-density-build --profile adversarial --corpus examples/llmwave-big-adversarial-density-corpus.json --out .nanda/llmwave-big-training/adversarial-density.json --format json
nanda-llmwave-big multi-profile-density-suite --rust-density .nanda/llmwave-big-training/strict-density.json --profile-evidence adversarial=.nanda/llmwave-big-training/adversarial-density.json --profile-evidence contracts=.nanda/llmwave-big-training/contracts-density.json --profile-evidence business=.nanda/llmwave-big-training/business-density.json --out .nanda/llmwave-big-training/multi-profile-density.json --format json
nanda-llmwave-big density-proof-doctor --suite .nanda/llmwave-big-training/multi-profile-density.json --out .nanda/llmwave-big-training/density-proof-doctor.json --format json
nanda-llmwave-big density-proof-doctor --suite .nanda/llmwave-big-training/multi-profile-density.json --min-fact-count 10 --out .nanda/llmwave-big-training/density-proof-doctor-medium.json --format json
nanda-llmwave-big density-ablation --suite .nanda/llmwave-big-training/multi-profile-density.json --format json
nanda-llmwave-big memory-final-proof --profile rust --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --format json
nanda-llmwave-big memory-final-proof --profile rust --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json --format json
nanda-llmwave-big memory-final-proof --profile rust --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json --heldout-eval .nanda/llmwave-big-training/rust-heldout-eval.json --format json
nanda-llmwave-big memory-final-proof --profile rust --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json --heldout-eval .nanda/llmwave-big-training/rust-heldout-eval.json --strict-density-evidence .nanda/llmwave-big-training/strict-density.json --format json
nanda-llmwave-big memory-final-proof --profile rust --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json --heldout-eval .nanda/llmwave-big-training/rust-heldout-eval.json --strict-density-evidence .nanda/llmwave-big-training/strict-density.json --multi-profile-density-evidence .nanda/llmwave-big-training/multi-profile-density.json --density-doctor-evidence .nanda/llmwave-big-training/density-proof-doctor.json --format json
nanda-llmwave-big nonlinear-memory-eval --format json
nanda-llmwave-big nonlinear-memory-eval \
  --corpus examples/llmwave-big-nonlinear-memory-corpus.json \
  --proof-policy strict-full-sweep \
  --format json
```

The ladder is the Phase 1 density instrument: it maps amortized wins,
standalone basis break-even, collision pressure, and the best operating window.
It intentionally keeps `nonlinear_memory_proven=false` until later phases and
the final proof gate pass.

`schema-residual-engine` is the Phase 2-3 controlled engine: it promotes
reused schema keys, writes matching facts as centroid updates plus compact
residuals, and keeps one-off facts as full fallbacks instead of forcing a bad
schema.

`memory-physics` is the Phase 4-5 collision/noise instrument. It runs clean,
collision, and noise trials over the schema-residual engine and applies
shortcut-specific 32-byte anti-wave records. The useful signal is false
positives before/after anti-wave, not a broad nonlinear-memory proof.

`memory-proof-path` is the Phase 6-8 bridge. It connects held-out inference,
basis economics from the density ladder, and route-balanced Wave Atlas memory.
It is still a controlled proof path, not the final big-corpus proof gate.

`memory-final-proof` is the Phase 9-12 command path. It checks field recall,
the LLMWave bridge, the big-corpus gate, and the final proof gate. The expected
honest state before a real big corpus is
`FINAL_PROOF_GATE_BLOCKED_BY_BIG_CORPUS`, with `nonlinear_memory_proven=false`.
Use `--profile rust` for the first code-oriented corpus: module owners, public
API exports, CLI dispatch, report printers, unit tests, integration tests, and
forbidden shortcuts such as "compiled command implies LLM readiness".
`rust-corpus-build` builds the first real Rust structural corpus artifact for
that profile. It closes only the artifact-building layer; held-out and focus
packets are still required before final proof claims.
`rust-heldout-build` consumes that artifact and writes withheld Rust route
questions plus negative shortcuts. It can make
`heldout_suite_ready=true`, but it still keeps focus, final proof, nonlinear
memory, and LLM claims closed.
`rust-focus-build` consumes the corpus and held-out suite and writes a
route-balanced focus packet. It caps dominant routes, removes exact held-out
facts from the training focus, and can make `focus_packet_ready=true`.
`memory-final-proof --profile rust --artifact ... --heldout-suite ...
--focus-packet ...` then stops at the next honest missing bridge:
`compile_test_evidence_bridge_missing`, not at missing corpus/focus.
`rust-compile-evidence-build` links saved `cargo check`, test, and clippy JSON
evidence to the focus packet. It can make
`compile_test_evidence_bridge_ready=true`; final proof should then move to
`rust_heldout_inference_eval_missing`, not to nonlinear-memory or LLM proof.
`rust-heldout-eval` consumes the focus packet plus held-out suite and runs
actual route-fact inference over withheld Rust questions. It reports held-out
pass rate and false-shortcut rejection. With both compile evidence and held-out
eval passed into final proof, the Rust profile can reach
`FINAL_PROOF_GATE_PROFILE_EVAL_READY_NOT_NONLINEAR_PROOF`; nonlinear-memory and
LLM claims still remain blocked by the strict density claim gate.
`strict-density-claim-gate` consumes the Rust corpus, focus packet, held-out
eval, and compile evidence. It compares packed profile bytes against the
linear fact baseline and checks schema reuse, residual saving, route balance,
held-out quality, false-shortcut rejection, and collision pressure. A pass is
`STRICT_DENSITY_PROFILE_PROVEN`, which is still a Rust profile claim only.
Passing that evidence into final proof moves the Rust profile to
`FINAL_PROOF_GATE_RUST_DENSITY_PROFILE_READY_NOT_GENERAL_LLM`; the general
nonlinear-memory and LLM claims remain false until multi-profile broad evals
exist.
`profile-density-build` adapts an independent relation corpus into the density
artifact consumed by `multi-profile-density-suite`. It is the generic non-Rust
path: a profile may pass as `PROFILE_DENSITY_PROVEN_NOT_GENERAL_LLM`, but it
does not prove general nonlinear memory or LLM readiness by itself.
`multi-profile-density-suite` aggregates independent density profile artifacts.
One Rust profile must return `MULTI_PROFILE_DENSITY_BLOCKED_BY_SINGLE_PROFILE`.
The general nonlinear-memory claim requires enough independent passing
profiles, held-out quality, false-shortcut rejection, and bounded collision
pressure. Profiles must also have distinct source signatures: duplicate
`source.corpus_hash` values or identical raw artifacts block the suite with
`duplicate_or_missing_independent_profile_sources`. A passing suite is not
enough for final proof by itself; `memory-final-proof` also requires
`--density-doctor-evidence`.
Use `examples/llmwave-big-adversarial-density-corpus.json` as an adversarial
profile when you need route-collision, namespace, near-root, and shortcut traps
in the density suite.
`density-proof-doctor` audits the strength of that suite. A suite can be
formally passing while the proof remains `DENSITY_PROOF_WEAK` because profiles
are tiny, source diversity is low, or adversarial/noise pressure is too mild.
WEAK/BLOCKED doctor evidence keeps the final nonlinear-memory claim closed.
For fixture-scale development, `--min-fact-count 10` can demonstrate a
`DENSITY_PROOF_MEDIUM` path over the business/contracts/adversarial suite. Treat
that as local medium evidence only: it may unlock
`FINAL_PROOF_GATE_NONLINEAR_MEMORY_READY_NOT_LLM`, never LLM readiness.
`density-ablation` reports suite-level profile criticality and the exposed
linear-baseline duel. It does not rerun field inference and does not prove
nonlinear memory by itself.

Scale-amortized mode is the local density result after fixed-basis overhead is
amortized. It does not unlock the general nonlinear-memory claim.

```bash
nanda-llmwave-big nonlinear-memory-eval \
  --corpus examples/llmwave-big-nonlinear-memory-corpus.json \
  --proof-policy scale-amortized \
  --format json
```

Expected boundary today:

```text
corpus_driven_memory.verdict = CORPUS_DRIVEN_AMORTIZED_DENSITY_OBSERVED
scale_amortized_nonlinear_memory_proven = true
nonlinear_memory_proven = false
```

Read `corpus_driven_memory` first when inspecting nonlinear memory. It is the
actual fixture-driven measurement: fact count, schema count, residual count,
linear bytes, standalone fixed-basis bytes, amortized fixed-basis bytes,
held-out pass rate, negative rejection, and noise rejection. A small corpus may
show an amortized win while still failing standalone strict density because the
64 KB basis overhead has not been repaid yet.

## Contract / Protocol Gate

Use this for contract, appendix, protocol-of-disagreements, EDI/EDO, and other
document-flow checks where role swaps are dangerous. The command is universal:
packet fields define parties, protocol direction, clauses, risk tags, and EDI
messages. Do not encode a project or counterparty in the implementation.

```bash
nanda-contract-gate --template --profile protocol --format json
nanda-contract-gate --input examples/contract-gate.protocol-pass.json --profile edo --format json
nanda-contract-gate --input examples/contract-gate.protocol-watch.json --profile protocol --format json
```

Read `STRUCTURAL_PASS_NOT_LEGAL_APPROVAL` literally. It means role/route/
protocol-direction coherence only. It is not permission to sign; final signing
still needs legal/accounting review.

## LLMWave Core Stages

```bash
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
```

## Word And Surface Memory

```bash
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
```

## Training, Hot Pack, And Small-Domain Eval

One-command local demo. It writes a tiny reproducible corpus and eval packet
under `.nanda/llmwave-big-demo`, then runs training, hot packing, scripted hot
chat eval, domain eval, and scale-amortized density eval. The expected verdict
is `DEMO_DOMAIN_PASS_NOT_BROAD_LLM`; this is not broad LLM readiness.

```bash
nanda-llmwave-big demo-domain --format json
```

Build a small project artifact:

```bash
mkdir -p .nanda/llmwave-big-training
nanda-llmwave-big train README.md CHANGELOG.md LLMWAVE_BIG_ROADMAP.md src examples \
  --out .nanda/llmwave-big-training/project-artifact.json \
  --format json
```

Ask and evaluate the artifact:

```bash
nanda-llmwave-big ask \
  --artifact .nanda/llmwave-big-training/project-artifact.json \
  --text "what does declaration require" \
  --top-k 5 \
  --format json
nanda-llmwave-big ask-eval \
  --artifact .nanda/llmwave-big-training/project-artifact.json \
  --suite examples/llmwave-big-ask-eval.json \
  --top-k 5 \
  --format json
```

Pack and query the hot core:

```bash
nanda-llmwave-big pack-hot \
  --artifact .nanda/llmwave-big-training/project-artifact.json \
  --out .nanda/llmwave-big-training/project.hot.bin \
  --format json
nanda-llmwave-big ask-hot \
  --hot-pack .nanda/llmwave-big-training/project.hot.bin \
  --artifact .nanda/llmwave-big-training/project-artifact.json \
  --text "invoice requires payment" \
  --top-k 5 \
  --format json
```

Scripted hot chat eval:

```bash
cat > .nanda/llmwave-big-training/chat.script <<'EOF'
ask broker requires invoice
learn accept: broker | requires | invoice
ask broker requires invoice
exit
EOF

nanda-llmwave-big chat-hot-eval \
  --hot-pack .nanda/llmwave-big-training/project.hot.bin \
  --artifact .nanda/llmwave-big-training/project-artifact.json \
  --memory .nanda/llmwave-big-training/chat-eval-memory.json \
  --script .nanda/llmwave-big-training/chat.script \
  --top-k 5 \
  --format json
```

Small-domain LLMWave eval:

```bash
nanda-llmwave-big domain-eval \
  --artifact .nanda/llmwave-big-training/project-artifact.json \
  --ask-suite examples/llmwave-big-ask-eval.json \
  --hot-pack .nanda/llmwave-big-training/project.hot.bin \
  --chat-script .nanda/llmwave-big-training/chat.script \
  --chat-memory .nanda/llmwave-big-training/domain-chat-memory.json \
  --nonlinear-corpus examples/llmwave-big-nonlinear-memory-corpus.json \
  --top-k 5 \
  --format json
```

## Public Corpus Helper

```bash
scripts/fetch-llmwave-big-gutenberg.sh
nanda-llmwave-big train README.md CHANGELOG.md LLMWAVE_BIG_ROADMAP.md src examples .nanda/external-corpus/gutenberg \
  --out .nanda/llmwave-big-training/project-gutenberg-artifact.json \
  --vocab-cap 65536 \
  --transition-cap 262144 \
  --active-chunk-cap 32768 \
  --chunk-tokens 64 \
  --format json
```

## Benchmarks

```bash
nanda-bench6m --mode active-core --support-build-iterations 1000 --format json
nanda-bench6m --mode write-density --support-build-iterations 1000 --format json
nanda-bench6m --mode consolidate --support-build-iterations 1000 --format json
nanda-bench6m --mode density --support-build-iterations 1000 --triads 15000 --format json
```
