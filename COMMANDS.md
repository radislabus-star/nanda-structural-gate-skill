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
nanda-llmwave-big linux-atlas-build --out-dir .nanda/linux-atlas --pack-kind base --max-facts 1000000 --format json
nanda-llmwave-big linux-atlas-build --out-dir .nanda/linux-atlas --pack-kind delta --max-facts 1000000 --format json
nanda-llmwave-big linux-active-field --atlas-dir .nanda/linux-atlas --max-active-facts 65536 --query "which package provides command bash" --format json
nanda-llmwave-big linux-pack-hot --atlas-dir .nanda/linux-atlas --max-active-facts 65536 --out .nanda/linux-active/linux-active-65k.laf --format json
nanda-llmwave-big linux-ask-hot --hot-pack .nanda/linux-active/linux-active-65k.laf --query "which package provides command bash" --top-k 5 --format json
nanda-llmwave-big linux-hot-eval --hot-pack .nanda/linux-active/linux-active-65k.laf --top-k 5 --format json
nanda-llmwave-big linux-domain-run --hot-pack .nanda/linux-active/linux-active-65k.laf --query "which package provides command bash" --top-k 5 --format json
nanda-llmwave-big linux-cache-proof --hot-pack .nanda/linux-active/linux-active-65k.laf --query "which package provides command bash" --iterations 64 --warmup-iterations 8 --samples 5 --format json
nanda-llmwave-big linux-pack-residual --atlas-dir .nanda/linux-atlas --max-active-facts 65536 --out .nanda/linux-active/linux-active-65k.lrf --format json
nanda-llmwave-big linux-residual-proof --residual-pack .nanda/linux-active/linux-active-65k.lrf --query "which package provides command bash" --top-k 5 --iterations 64 --warmup-iterations 8 --samples 5 --format json
nanda-llmwave-big linux-exposure-run --residual-pack .nanda/linux-active/linux-active-65k.lrf --max-candidates 16 --format json
nanda-llmwave-big linux-snapshot-import --snapshot .nanda/linux-active/runtime-snapshot.json --format json
nanda-llmwave-big linux-exposure-run --residual-pack .nanda/linux-active/linux-active-65k.lrf --runtime-snapshot .nanda/linux-active/runtime-snapshot.json --max-candidates 16 --format json
nanda-llmwave-big linux-chat-run --residual-pack .nanda/linux-active/linux-active-65k.lrf --max-facts 4 --format json
nanda-llmwave-big linux-chat-v1-eval --residual-pack .nanda/linux-active/linux-active-65k.lrf --max-facts 4 --format json
nanda-llmwave-big linux-chat-v1 --residual-pack .nanda/linux-active/linux-active-65k.lrf --prompt "Which package provides command bash?" --prompt "I meant systemctl." --max-facts 4 --format json
nanda-llmwave-big linux-chat-v1 --residual-pack .nanda/linux-active/linux-active-65k.lrf --script .nanda/linux-active/linux-chat.script --max-facts 4 --format json
nanda-llmwave-big linux-chat-v2-eval --residual-pack .nanda/linux-active/linux-active-65k.lrf --memory .nanda/linux-active/linux-chat-v2-eval.lwm --max-facts 4 --format json
nanda-llmwave-big linux-chat-v2 --residual-pack .nanda/linux-active/linux-active-65k.lrf --memory .nanda/linux-active/linux-chat-v2.lwm --prompt "Which package provides command foocmd?" --prompt "learn accept: foocmd | linux.apt.command.provider | foopkg" --prompt "Which package provides command foocmd?" --max-facts 4 --format json
nanda-llmwave-big linux-vpn-train --memory .nanda/linux-active/linux-vpn.lwm --reset-memory --format json
nanda-llmwave-big linux-vpn-train-eval --residual-pack .nanda/linux-active/linux-active-65k.lrf --memory .nanda/linux-active/linux-vpn-eval.lwm --max-facts 4 --format json
nanda-llmwave-big linux-query-wave --text "Is ssh externally exposed?" --format json
nanda-llmwave-big linux-reason-run --residual-pack .nanda/linux-active/linux-active-65k.lrf --text "Is ssh externally exposed?" --max-facts 4 --format json
nanda-llmwave-big linux-reason-run --residual-pack .nanda/linux-active/linux-active-65k.lrf --runtime-snapshot .nanda/linux-active/runtime-snapshot.json --text "Is ssh externally exposed?" --max-facts 4 --format json
nanda-llmwave-big linux-broad-suite-build --residual-pack .nanda/linux-active/linux-active-65k.lrf --cases 100 --out .nanda/linux-active/linux-broad-suite.json --format json
nanda-llmwave-big linux-broad-eval-run --residual-pack .nanda/linux-active/linux-active-65k.lrf --suite .nanda/linux-active/linux-broad-suite.json --out .nanda/linux-active/linux-broad-eval.json --max-facts 4 --format json
nanda-llmwave-big linux-profile-claim-gate --residual-pack .nanda/linux-active/linux-active-65k.lrf --broad-eval .nanda/linux-active/linux-broad-eval.json --format json
nanda-llmwave-big linux-heldout-suite-build --residual-pack .nanda/linux-active/linux-active-65k.lrf --cases 100 --out .nanda/linux-active/linux-heldout-suite.json --format json
nanda-llmwave-big linux-heldout-eval-run --residual-pack .nanda/linux-active/linux-active-65k.lrf --suite .nanda/linux-active/linux-heldout-suite.json --out .nanda/linux-active/linux-heldout-eval.json --max-facts 4 --format json
nanda-llmwave-big linux-feedback-build --residual-pack .nanda/linux-active/linux-active-65k.lrf --text "Is this machine externally exposed?" --decision reject --out .nanda/linux-active/linux-feedback.json --format json
nanda-llmwave-big linux-feedback-apply --residual-pack .nanda/linux-active/linux-active-65k.lrf --feedback .nanda/linux-active/linux-feedback.json --text "Is this machine externally exposed?" --max-facts 4 --format json
nanda-llmwave-big linux-decision-search --residual-pack .nanda/linux-active/linux-active-65k.lrf --text "Is this machine externally exposed?" --max-facts 4 --format json
nanda-llmwave-big linux-decision-search --residual-pack .nanda/linux-active/linux-active-65k.lrf --runtime-snapshot .nanda/linux-active/runtime-snapshot.json --text "Is this machine externally exposed?" --max-facts 4 --format json
nanda-llmwave-big linux-relation-profile --residual-pack .nanda/linux-active/linux-active-65k.lrf --format json
nanda-llmwave-big security-fixture-run --format json
nanda-llmwave-big daybreak-duel --format json
nanda-llmwave-big strict-density-claim-gate --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --heldout-eval .nanda/llmwave-big-training/rust-heldout-eval.json --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json --out .nanda/llmwave-big-training/strict-density.json --format json
nanda-llmwave-big profile-density-build --profile business --corpus examples/llmwave-big-nonlinear-memory-corpus.json --out .nanda/llmwave-big-training/business-density.json --format json
nanda-llmwave-big profile-density-build --profile contracts --corpus examples/llmwave-big-contract-density-corpus.json --out .nanda/llmwave-big-training/contracts-density.json --format json
nanda-llmwave-big profile-density-build --profile adversarial --corpus examples/llmwave-big-adversarial-density-corpus.json --out .nanda/llmwave-big-training/adversarial-density.json --format json
nanda-llmwave-big multi-profile-density-suite --rust-density .nanda/llmwave-big-training/strict-density.json --profile-evidence adversarial=.nanda/llmwave-big-training/adversarial-density.json --profile-evidence contracts=.nanda/llmwave-big-training/contracts-density.json --profile-evidence business=.nanda/llmwave-big-training/business-density.json --out .nanda/llmwave-big-training/multi-profile-density.json --format json
nanda-llmwave-big density-proof-doctor --suite .nanda/llmwave-big-training/multi-profile-density.json --out .nanda/llmwave-big-training/density-proof-doctor.json --format json
nanda-llmwave-big density-proof-doctor --suite .nanda/llmwave-big-training/multi-profile-density.json --min-fact-count 10 --out .nanda/llmwave-big-training/density-proof-doctor-medium.json --format json
nanda-llmwave-big density-ablation --suite .nanda/llmwave-big-training/multi-profile-density.json --out-hot-packet .nanda/llmwave-big-training/density-ablation.hot --format json
nanda-llmwave-big broad-corpus-build --out .nanda/llmwave-big-training/broad-corpus.json --format json
nanda-llmwave-big broad-corpus-build --source examples/llmwave-big-broad-public-corpus.txt --profile public-safe-strong-seed --out .nanda/llmwave-big-training/broad-public-corpus.json --format json
nanda-llmwave-big broad-corpus-build --source examples/llmwave-big-broad-public-corpus-100k.txt --profile public-safe-100k --out .nanda/llmwave-big-training/broad-public-100k-corpus.json --format json
scripts/build-llmwave-big-broad-public-corpus.sh .nanda/llmwave-big-corpus/public-safe-1m.txt .nanda/llmwave-big-corpus/public-safe-1m.manifest.json 1000000
nanda-llmwave-big broad-corpus-build --source .nanda/llmwave-big-corpus/public-safe-1m.txt --profile public-safe-1m --out .nanda/llmwave-big-corpus/public-safe-1m.corpus.json --format json
nanda-llmwave-big broad-heldout-build --corpus .nanda/llmwave-big-corpus/public-safe-1m.corpus.json --out .nanda/llmwave-big-corpus/public-safe-1m.heldout.json --max-cases 1024 --format json
nanda-llmwave-big broad-focus-build --corpus .nanda/llmwave-big-corpus/public-safe-1m.corpus.json --heldout-suite .nanda/llmwave-big-corpus/public-safe-1m.heldout.json --out .nanda/llmwave-big-corpus/public-safe-1m.focus.json --max-facts 15000 --route-fact-cap 300 --format json
nanda-llmwave-big broad-eval-run --corpus .nanda/llmwave-big-corpus/public-safe-1m.corpus.json --suite .nanda/llmwave-big-corpus/public-safe-1m.heldout.json --focus-packet .nanda/llmwave-big-corpus/public-safe-1m.focus.json --hot-packet .nanda/llmwave-big-training/density-ablation.hot --out .nanda/llmwave-big-corpus/public-safe-1m.broad-eval.json --format json
nanda-llmwave-big broad-dataset-doctor --corpus .nanda/llmwave-big-training/broad-corpus.json --out .nanda/llmwave-big-training/broad-dataset-doctor.json --format json
nanda-llmwave-big broad-eval-suite-build --corpus .nanda/llmwave-big-training/broad-corpus.json --out .nanda/llmwave-big-training/broad-eval-suite.json --format json
nanda-llmwave-big broad-heldout-build --corpus .nanda/llmwave-big-training/broad-corpus.json --out .nanda/llmwave-big-training/broad-heldout.json --format json
nanda-llmwave-big broad-focus-build --corpus .nanda/llmwave-big-training/broad-corpus.json --heldout-suite .nanda/llmwave-big-training/broad-heldout.json --out .nanda/llmwave-big-training/broad-focus.json --format json
nanda-llmwave-big broad-eval-run --corpus .nanda/llmwave-big-training/broad-corpus.json --suite .nanda/llmwave-big-training/broad-heldout.json --focus-packet .nanda/llmwave-big-training/broad-focus.json --hot-packet .nanda/llmwave-big-training/density-ablation.hot --out .nanda/llmwave-big-training/broad-eval.json --format json
nanda-llmwave-big broad-baseline-duel --eval-report .nanda/llmwave-big-training/broad-eval.json --out .nanda/llmwave-big-training/broad-baseline-duel.json --format json
nanda-llmwave-big broad-chat-loop-eval --out .nanda/llmwave-big-training/broad-chat-loop.json --format json
nanda-llmwave-big llmwave-readiness --memory-final-proof .nanda/llmwave-big-training/memory-final-proof.json --broad-dataset-doctor .nanda/llmwave-big-training/broad-dataset-doctor.json --broad-eval .nanda/llmwave-big-training/broad-eval.json --baseline-duel .nanda/llmwave-big-training/broad-baseline-duel.json --chat-loop .nanda/llmwave-big-training/broad-chat-loop.json --out .nanda/llmwave-big-training/llmwave-readiness.json --format json
nanda-llmwave-big memory-final-proof --profile rust --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --format json
nanda-llmwave-big memory-final-proof --profile rust --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json --format json
nanda-llmwave-big memory-final-proof --profile rust --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json --heldout-eval .nanda/llmwave-big-training/rust-heldout-eval.json --format json
nanda-llmwave-big memory-final-proof --profile rust --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json --heldout-eval .nanda/llmwave-big-training/rust-heldout-eval.json --strict-density-evidence .nanda/llmwave-big-training/strict-density.json --format json
nanda-llmwave-big memory-final-proof --profile rust --artifact .nanda/llmwave-big-training/rust-corpus-artifact.json --heldout-suite .nanda/llmwave-big-training/rust-heldout-suite.json --focus-packet .nanda/llmwave-big-training/rust-focus-packet.json --compile-evidence .nanda/llmwave-big-training/rust-compile-evidence.json --heldout-eval .nanda/llmwave-big-training/rust-heldout-eval.json --strict-density-evidence .nanda/llmwave-big-training/strict-density.json --multi-profile-density-evidence .nanda/llmwave-big-training/multi-profile-density.json --density-doctor-evidence .nanda/llmwave-big-training/density-proof-doctor.json --density-hot-packet .nanda/llmwave-big-training/density-ablation.hot --format json
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
`linux-atlas-build` starts the Linux SysField data path. It reads local Linux
metadata without secret-bearing config scans: dpkg package status, package file
lists, manpage index, systemd unit fields, `/etc/os-release`, resolver/routes,
socket summaries, and built-in negative boundary facts. The output is
append-only under `.nanda/linux-atlas/`: `facts/base-*.jsonl` or
`facts/delta-*.jsonl`, `manifest.json`, `consolidated/linux-atlas-current.json`,
route indexes, `indexes/fact-ids.txt`, and `packs.jsonl`. Delta packs skip
previously known fact IDs and write only new facts discovered after the base
pack. Treat `LINUX_ATLAS_BASE_READY` as a knowledge-atlas result only; active
65k packing, exposure analysis, LLM readiness, and nonlinear-memory proof remain
closed.
`linux-active-field` is the first active projection over that atlas. It reads
the append-only fact packs, builds a route-balanced active window capped by
`--max-active-facts` (default 65,536), estimates the 64-byte packed projection
budget, and runs review-grade Linux probes such as command provider lookup,
systemd exec hints, and negative boundary checks. Treat
`LINUX_ACTIVE_FIELD_READY_NOT_LLM` as active-field readiness only: binary hot
packet execution, exposure analysis, broad eval, LLM readiness, and nonlinear
memory proof remain blocked.
`linux-pack-hot` materializes that active window as a `.laf` binary packet with
64-byte fixed records. The report separates hot-loop record bytes from the cold
label table used for explanation. Treat
`LINUX_HOT_PACKET_READY_NOT_CACHE_ONLY_PROOF` as binary fixed-record readiness:
it does not prove full cache-only execution, exposure analysis, LLM readiness,
or nonlinear memory.
`linux-ask-hot` scans the `.laf` fixed records and uses the embedded cold label
table only to explain the top facts. Treat `LINUX_HOT_SCAN_READY_NOT_LLM` as a
Linux hot-field query result, not a general answer engine.
`linux-hot-eval` runs built-in Linux domain probes over that packet and compares
the field against a lexical overlap baseline. Treat
`LINUX_HOT_EVAL_PASS_NOT_LLM` as a local Linux-domain eval gate only.
`linux-domain-run` ties the Linux query wave, fixed-record hot scan, route peak,
lexical duel, constrained answer surface, verifier, and feedback preview into
one operator-facing report. Treat
`LINUX_DOMAIN_LLMWAVE_READY_NOT_GENERAL_LLM` as constrained Linux-domain
readiness only.
`linux-cache-proof` reads only the `.laf` header plus fixed-record section,
excludes the cold label table, compiles the query to numeric hash anchors, and
benchmarks repeated full scans of fixed 64-byte records. Treat
`LINUX_CACHE_ONLY_EXECUTION_PROVEN` as software cache-budget runtime proof for
the hot loop: no JSON, labels, file I/O, heap allocation, or per-record score
arrays in the measured loop. It is not broad chat readiness, nonlinear-memory
proof, exposure analysis, or hardware PMU cache-miss proof.
`linux-pack-residual` materializes the same Linux Active Field as a `.lrf`
binary schema/residual memory: repeated `route+relation+polarity` modes become
`SchemaRecord32`, subject/object variation becomes `ResidualRecord32`, and
one-off facts stay as `FallbackRecord64`. Treat
`LINUX_SCHEMA_RESIDUAL_PACKET_READY_NOT_PROOF` as written binary memory only.
`linux-residual-proof` scans the `.lrf` schema/residual/fallback sections,
runs the Linux domain eval and lexical duel, and compares the actual binary hot
section bytes against a direct fixed64 baseline. Treat
`LINUX_SCHEMA_RESIDUAL_MEMORY_PROVEN` as a Linux-profile nonlinear memory proof:
real binary schema/residual storage beats fixed64 bytes and preserves the
Linux-domain eval. It is not broad chat readiness or exposure reasoning.
`linux-exposure-run` reads the `.lrf` schema/residual memory and builds a
boundary-aware Linux exposure field: local sockets, bind scope, firewall allow
facts, service context, and negative boundary facts. Treat
`LINUX_EXPOSURE_REASONING_READY_NOT_SCANNER` as exposure-reasoning readiness
only. It is not a network scan, exploit, vulnerability proof, or broad chat
LLM.
`linux-snapshot-import` converts a user-provided JSON runtime snapshot into
temporary Linux-profile facts. It never runs `ss`, `nft`, `ufw`, `iptables`,
`systemctl`, or any other runtime command. Pass the same snapshot to
`linux-exposure-run`, `linux-reason-run`, or `linux-decision-search` with
`--runtime-snapshot` when the `.lrf` packet needs side-effect-free runtime
evidence such as firewall allow rules, listeners, or service state. This overlay
does not rewrite hot memory and does not prove exposure by itself.
`linux-chat-run` reads the same `.lrf` schema/residual memory and runs a
constrained Linux-profile multi-turn readout: grounded package/provider answers,
listener/exposure boundary answers, context recall, and false-shortcut refusal.
Treat `LINUX_PROFILE_BROAD_CHAT_READY_NOT_GENERAL_LLM` as Linux-profile chat
readiness only. It is not open-domain chat, not a general GPT-style model, not a
network scanner, and not a vulnerability proof.
`linux-chat-v1` is the bounded chat-loop surface on top of the same reasoning
owner. It keeps short dialogue state, resolves constrained follow-ups and
corrections, delegates evidence-chain decisions to `linux-reason-run` logic, and
keeps anti-wave shortcut refusal active. `linux-chat-v1-eval` runs the built-in
script for provider answer, correction, listener follow-up, exposure refusal,
vulnerability shortcut refusal, and unsupported-prompt refusal. Treat
`LINUX_CHAT_V1_READY_NOT_GENERAL_LLM` as bounded Linux-profile chat readiness
only.
`linux-chat-v2` adds persistent wave-memory learning from dialogue feedback. It
writes explicit accept/reject/learn feedback as fixed 32-byte `.lwm` wave-delta
records, reloads that memory before the next question, and proves that a later
field pass can change because of memory rather than transcript replay.
`linux-chat-v2-eval` checks positive memory lift, learned anti-wave replay, and
unrelated-route preservation. Treat
`LINUX_CHAT_V2_PERSISTENT_WAVE_LEARNING_READY_NOT_GENERAL_LLM` as local
Linux-profile dialogue learning only, not general LLM readiness and not a final
nonlinear-memory proof.
`linux-vpn-train` writes a safe local VPN training profile into persistent
wave memory: WireGuard setup, status checks, DNS/routes, NetworkManager import,
TrustTunnel safety, and secret boundaries. It does not mutate the local system,
read secret files, or print private keys. `linux-vpn-train-eval` trains that
memory and proves through `linux-chat-v2` that VPN questions answer from wave
memory while secret-material requests activate learned anti-wave refusal.
`linux-query-wave` compiles one Linux-profile prompt into intent, anchors,
route priors, negative boundaries, forbidden shortcuts, and answer policy. It is
input shaping only, not retrieval.
`linux-reason-run` applies that query wave to `.lrf` schema/residual memory,
builds an evidence chain, applies anti-wave shortcut suppression, and returns a
grounded decision for that one prompt.
`linux-broad-suite-build` generates a Linux-profile eval suite from the active
facts; `linux-broad-eval-run` executes the suite against `.lrf`; and
`linux-profile-claim-gate` only allows
`LINUX_PROFILE_REASONING_READY_NOT_GENERAL_LLM` when schema/residual memory
proof and broad Linux-profile eval thresholds both pass. These commands expand
the Linux-profile reasoning proof surface, but they still do not unlock general
LLM, open-domain chat, scanner, or exploit claims.
`linux-heldout-suite-build` adds a stricter profile suite: exact facts,
near-name collisions, shortcut controls, and endpoint-scope checks. Use
`linux-heldout-eval-run` before trusting the profile on noisy Linux facts.
`linux-feedback-build` writes a local profile memory packet from accept/reject
decisions, and `linux-feedback-apply` shows how the packet changes the next
field pass through learned positive lanes or anti-wave lanes. This is local
feedback, not gradient training.
`linux-decision-search` turns a Linux question into missing evidence and
side-effect-free next checks. It proposes commands such as `ss`, `systemctl`,
`nft`, or `ip` as evidence routes, but it does not run them and is not a
scanner. If a runtime snapshot already closes the missing evidence route, the
state becomes `ANSWER_ALREADY_GROUNDED` instead of proposing redundant checks.
`linux-relation-profile` reports relation-family coverage and missing routes
over the `.lrf` packet so the corpus can grow by causal relation type instead
of raw fact count alone.
`security-fixture-run` runs the first concrete safe find -> patch -> verify
loop. It creates a temporary toy path-traversal fixture, proves the vulnerable
version can read `../secret.txt`, emits a canonicalize-and-prefix patch
candidate, verifies the patched version blocks escape, and verifies the normal
file still reads. Treat `DEFENSIVE_PATCH_PROVEN_LOCAL_FIXTURE` as local fixture
proof only, not a real-project scanner or exploit generator.
`daybreak-duel` runs a safe Daybreak-style defensive baseline over local
fixtures. It checks shortcut rejection, side-effect-free runtime snapshot
evidence, grounded decision-search stopping, and the local patch fixture loop,
then keeps real-project remediation verification explicitly blocked. Treat
`DAYBREAK_DUEL_BASELINE_READY_NOT_COMPETITIVE` as a scoreboard, not a claim
that NANDA matches GPT-5.5-Cyber/Daybreak.
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
nonlinear memory by itself. Its JSON output is still wrapped in the unified
field projection, so agents can compare it with other field-aware reports.
The report also exposes a read-only `runtime_path`: L2 profile surfaces plus L3
proof axes. `hot_loop_ready=false` there is intentional until a binary active
packet exists.
With `--out-hot-packet`, the command writes a compact binary packet with a
16-byte header and 16-byte fixed records. This proves artifact materialization,
not hot-loop execution.
Pass that packet to `memory-final-proof` with `--density-hot-packet` to mark the
binary density evidence as present in final proof. It still does not make
`llm_ready=true`.

`broad-corpus-build`, `broad-dataset-doctor`, `broad-heldout-build`,
`broad-focus-build`, `broad-eval-run`, `broad-baseline-duel`,
`broad-chat-loop-eval`, and `llmwave-readiness` are the broad cognition path.
They test corpus quality, semantic diversity, exact held-out removal,
domain-route-balanced focus, near-duplicate leakage, recall, role binding, route
reasoning, multi-hop context, answer generation, adversarial shortcut rejection,
feedback, baseline duels, and constrained multi-turn correction/refusal.
`broad-eval-suite-build` remains useful as a controlled smoke fixture; the
external path should use held-out/focus artifacts. A full external-medium pass can open
`LLMWAVE_READY_CANDIDATE_EXTERNAL_MEDIUM`, but it still keeps
`llm_ready=false`; this is a candidate boundary, not a general LLM proof.
`examples/llmwave-big-broad-public-corpus-100k.txt` is the normal public-safe
large seed for this path: 100,000 generated route facts, 10 domains, 50 routes,
and no user/private business data. The smaller
`examples/llmwave-big-broad-public-corpus.txt` remains a 96-fact smoke seed.
For the local 1M stress corpus, generate under ignored `.nanda/` with
`scripts/build-llmwave-big-broad-public-corpus.sh ... 1000000`. On the current
machine this produced a 224 MiB text source and a 382 MiB JSON artifact;
`broad-corpus-build` took about 3.8 seconds with about 2.1 GiB peak RSS, and
`broad-dataset-doctor` took about 1.7 seconds with about 0.9 GiB peak RSS.
The 1M held-out and focus builders are domain-route-balanced: reports must show
domain coverage, route coverage, family coverage, and near-duplicate leakage
before readiness is trusted. Current local 1M measurement: 1024 held-out cases
cover 10 domains, 50 routes, and 8 eval families; the 15,000 fact focus packet
has domain balance 1.0, route balance 1.0, and near-duplicate leakage 0. The
full local 1M path can reach `LLMWAVE_READY_CANDIDATE_EXTERNAL_STRONG`, but only
with `llm_ready=false`.

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
nanda-llmwave-big core-v1-contract --format json
nanda-llmwave-big core-v1-field-cutover --format json
nanda-llmwave-big core-v1-memory-writer --format json
nanda-llmwave-big core-v1-nonlinear-proof --format json
nanda-llmwave-big core-v1-query-wave --text "Has customs cleared the goods?" --format json
nanda-llmwave-big core-v1-active-retrieval --text "Has customs cleared the goods?" --format json
nanda-llmwave-big core-v1-schema-reasoning --text "Has customs cleared the goods?" --format json
nanda-llmwave-big core-v1-surface-generation --text "Has customs cleared the goods?" --format json
nanda-llmwave-big core-v1-answer-verifier --text "Has customs cleared the goods?" --format json
nanda-llmwave-big core-v1-feedback-learning --text "Has customs cleared the goods?" --format json
nanda-llmwave-big core-v1-consolidation-sleep --text "Has customs cleared the goods?" --format json
nanda-llmwave-big core-v1-broad-eval --text "Has customs cleared the goods?" --format json
nanda-llmwave-big core-v2-contract --format json
nanda-llmwave-big core-v2-corpus --format json
nanda-llmwave-big core-v2-heldout --format json
nanda-llmwave-big core-v2-focus --format json
nanda-llmwave-big core-v2-density --format json
nanda-llmwave-big core-v2-run --text "Has customs cleared the goods?" --format json
nanda-llmwave-big core-v2-pack-hot --format json
nanda-llmwave-big core-v2-claim-gate --format json
nanda-llmwave-big core-v3-plan --format json
nanda-llmwave-big core-v3-solution-search --goal "confirm customs clearance" --format json
nanda-llmwave-big core-v3-pack-1m --format json
nanda-llmwave-big core-v3-claim-gate --goal "confirm customs clearance" --format json
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

`core-v1-contract` is the Phase 1 LLMWave Core V1 contract from
`LLMWAVE_CORE_V1_CONTRACT.md`. It defines the full model loop, component
owners, required boundaries, and claim table before implementation phases start.
It deliberately reports `CORE_V1_CONTRACT_RECORDED_NOT_IMPLEMENTED`; it is not
an LLM readiness proof.
`core-v1-field-cutover` is the Phase 2 report from
`LLMWAVE_CORE_V1_PHASE2_REPORT.md`. It records `field_core` as the sole owner
of shared field operations while keeping `field_core_as_sole_llmwave_core_engine`,
`nonlinear_memory_proven`, and `llm_ready` false.
`core-v1-memory-writer` is the Phase 3 report from
`LLMWAVE_CORE_V1_PHASE3_REPORT.md`. It records the schema residual plus surface
family memory writer and keeps nonlinear memory proof closed until Phase 4.
`core-v1-nonlinear-proof` is the Phase 4 report from
`LLMWAVE_CORE_V1_PHASE4_REPORT.md`. It may mark a nonlinear-memory candidate,
but it must keep `nonlinear_memory_proven=false` until held-out, external,
leakage, and broad-noise gates are bound to the writer path.
`core-v1-query-wave` is the Phase 5 report from
`LLMWAVE_CORE_V1_PHASE5_REPORT.md`. It converts user text into a structured
query wave with route, role, operator, evidence, time/currentness, uncertainty,
and polarity components. It keeps `safe_to_answer=false`: the output is a
field input, not an answer.
`core-v1-active-retrieval` is the Phase 6 report from
`LLMWAVE_CORE_V1_PHASE6_REPORT.md`. It runs the query wave through route peaks,
local focus, and a field pass. A focused route can be retrieval-ready for the
next phase, but contested, thin, reversed, noisy, and no-answer states still
block answer generation.
`core-v1-schema-reasoning` is the Phase 7 report from
`LLMWAVE_CORE_V1_PHASE7_REPORT.md`. It turns a focused field peak into an
explicit schema answer plan and keeps surface generation closed until Phase 8.
`core-v1-surface-generation` is the Phase 8 report from
`LLMWAVE_CORE_V1_PHASE8_REPORT.md`. It materializes constrained evidence-bound
answer surfaces and leaves final answer permission to the Phase 9 verifier.
`core-v1-answer-verifier` is the Phase 9 report from
`LLMWAVE_CORE_V1_PHASE9_REPORT.md`. It verifies local evidence-bound answer
surfaces and blocks unsupported positive claims, role swaps, and WATCH/split
surfaces before feedback learning starts.
`core-v1-feedback-learning` is the Phase 10 report from
`LLMWAVE_CORE_V1_PHASE10_REPORT.md`. It emits shortcut-specific feedback
memory and shows the next local field pass changing while keeping consolidation,
broad learning, LLM readiness, and nonlinear-memory proof closed.
`core-v1-consolidation-sleep` is the Phase 11 report from
`LLMWAVE_CORE_V1_PHASE11_REPORT.md`. It merges local feedback safely, preserves
negative shortcut memory, and keeps broad eval/training and hard claims closed.
`core-v1-broad-eval` is the Phase 12 report from
`LLMWAVE_CORE_V1_PHASE12_REPORT.md`. It runs embedded broad controls for the
Core V1 local pipeline and explicitly keeps real broad-corpus generalization,
LLM readiness, and nonlinear-memory proof blocked.
`core-v2-contract`, `core-v2-corpus`, `core-v2-heldout`, `core-v2-focus`,
`core-v2-density`, `core-v2-run`, `core-v2-pack-hot`, and
`core-v2-claim-gate` are the Core V2 staged pipeline. They move from a local
public-safe relation fixture to held-out removal, route-balanced focus, density
candidate checks, a local evidence-bound route run, and a compact hot-packet
storage report. The passing claim gate is
`CORE_V2_LOCAL_PIPELINE_READY_NOT_LLM`; it deliberately keeps general LLM,
nonlinear-memory proof, real broad corpus, and cache-only execution claims
closed.

`core-v3-plan`, `core-v3-solution-search`, `core-v3-pack-1m`, and
`core-v3-claim-gate` are the Core V3 Goal/Action/Constraint/Solution Search
pipeline. It adds fixed records for goals, actions, constraints, solution
steps, and 1M active projection records. `core-v3-solution-search` answers
"which steps are needed so this can become possible" instead of emitting an
unsupported yes/no. `core-v3-pack-1m` reads the local public-safe 1M manifest,
route-balanced 15k focus packet, and held-out suite, then counts the 6 MiB
active projection budget. The passing claim gate is
`CORE_V3_SOLUTION_AND_1M_PROJECTION_READY_NOT_LLM`; it deliberately keeps
general LLM, final nonlinear-memory proof, cache-only execution, and lossless
million-fact hot storage claims closed.

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
nanda-bench6m --mode active-65k --active-65k-iterations 1 --format json
nanda-bench6m --mode active-core --support-build-iterations 1000 --format json
nanda-bench6m --mode write-density --support-build-iterations 1000 --format json
nanda-bench6m --mode consolidate --support-build-iterations 1000 --format json
nanda-bench6m --mode density --support-build-iterations 1000 --triads 65536 --format json
nanda-llmwave-big claim-gate --claim active-65k-runtime --format json
```
