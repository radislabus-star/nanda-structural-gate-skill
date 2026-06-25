# LLMWave Core V3 Report

Core V3 adds a Goal/Action/Constraint/Solution Search Field and a 1M active
projection gate over the local public-safe broad corpus artifact.

It is a step from "memory says PASS/VETO/WATCH" toward "the field can return
which missing actions and evidence would make the goal possible".

## Commands

```bash
nanda-llmwave-big core-v3-plan --format json
nanda-llmwave-big core-v3-solution-search \
  --goal "confirm customs clearance" \
  --format json
nanda-llmwave-big core-v3-pack-1m --format json
nanda-llmwave-big core-v3-claim-gate \
  --goal "confirm customs clearance" \
  --format json
```

## Goal/Action/Constraint/Solution Field

The new fixed records are:

- `CoreV3GoalRecord32`
- `CoreV3ActionRecord32`
- `CoreV3ConstraintRecord32`
- `CoreV3SolutionStepRecord64`
- `CoreV3MillionPackRecord32`

The solution-search fixture is intentionally narrow and evidence-bound. For the
goal `confirm customs clearance`, Core V3 does not answer "yes". It returns:

1. reject the invoice-only shortcut;
2. request the declaration packet;
3. request release or clearance evidence;
4. answer only the needed steps until the final fact is proven.

Expected state:

```text
CORE_V3_SOLUTION_SEARCH_READY_NOT_GENERAL_REASONER
SOLUTION_PATH_FOUND_MISSING_EVIDENCE
final_fact_confirmed=false
```

## 1M Active Projection

The local broad corpus artifact is under:

```text
.nanda/llmwave-big-corpus/public-safe-1m.manifest.json
.nanda/llmwave-big-corpus/public-safe-1m.focus.json
.nanda/llmwave-big-corpus/public-safe-1m.heldout.json
```

Core V3 reads the manifest, the route-balanced focus packet, and the held-out
suite. It counts only the active hot projection:

- fixed basis;
- route centroids;
- domain-route centroids;
- 15,000 focused 32-byte records;
- held-out constraint guards;
- 1M fact signature bitset;
- solution-search field;
- action/constraint lane space.

Expected current budget:

```text
facts:              1,000,000
focus records:         15,000
hot budget:         6,291,456 bytes
used:               2,049,224 bytes
verdict:            CORE_V3_1M_ACTIVE_PROJECTION_FITS_6M_NOT_LOSSLESS_STORAGE
```

This is not lossless storage of one million full facts inside 6 MiB. The full
corpus remains cold/warm Atlas data. The hot core stores signatures, centroids,
guards, and the active focus window.

## Claim Boundary

The passing Core V3 claim gate is:

```text
CORE_V3_SOLUTION_AND_1M_PROJECTION_READY_NOT_LLM
```

It means:

- local solution-search field is wired;
- public-safe 1M active projection budget fits 6 MiB;
- the system can explain which steps are missing for a constrained goal.

It does not mean:

- `llm_ready=true`;
- `nonlinear_memory_proven=true`;
- `cache_only_execution_proven=true`;
- `lossless_million_fact_hot_storage=true`;
- broad chat/generation/general reasoning is solved.

Those claims remain closed until broader eval, cache-only profiling, and
multi-profile nonlinear-memory proof bind back to this runtime.
