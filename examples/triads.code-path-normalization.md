# NANDA Code Path Normalization Example

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| t1 | src/core/gate.rs | exposes | src/bin/nanda.rs | src/main.rs:10 | 0.9 | module | binary | compile | code-flow |
| t2 | src/core/gate.rs | calls | src/core/wave.rs | src/core/gate.rs:40 | 0.9 | module | module | runtime | code-flow |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | core::gate | exposes | bin::nanda | candidate_answer | 0.9 | module | binary | compile | candidate-code-flow |
| c2 | core::gate | calls | core::wave | candidate_answer | 0.9 | module | module | runtime | candidate-code-flow |
