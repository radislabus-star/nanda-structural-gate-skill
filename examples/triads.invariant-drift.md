# Invariant Drift Example

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| t1 | setting.timeout | default_value | 500ms | source.rs:1 | 0.9 | contract | value | config | timeout-contract |
| t2 | setting.timeout | default_value | 500ms | runtime.rs:1 | 0.9 | implementation | value | config | timeout-contract |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | setting.timeout | default_value | 300ms | ui.js:1 | 0.9 | consumer | value | config | candidate-timeout-contract |
