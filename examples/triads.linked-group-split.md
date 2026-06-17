# Linked Group Split Example

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| t1 | source module A | exposes | CLI A | src/a.rs:1 | 0.9 | source | cli | execution | flow-a |
| t2 | CLI A | calls | runtime A | src/a.rs:2 | 0.9 | cli | runtime | execution | flow-a |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | source module A | exposes | CLI A | candidate_answer | 0.9 | source | cli | execution | candidate-flow-a |
| c2 | CLI A | calls | runtime A | candidate_answer | 0.9 | cli | runtime | execution | candidate-flow-a |
