# Skill Flow Splice Example

## triads

| id | subject                | relation | object         | evidence                    | confidence | subject_role | object_role | route     | group        |
|----|------------------------|----------|----------------|-----------------------------|------------|--------------|-------------|-----------|--------------|
| a1 | source skill A         | syncs_to | runtime skill A| install-a.sh:10             |        0.9 | source       | runtime     | deploy    | skill-flow-a |
| a2 | runtime skill A        | exposes  | command A      | install-a.sh:15             |        0.9 | runtime      | cli         | deploy    | skill-flow-a |
| a3 | command A              | returns  | verdict A      | command-a:1                 |        0.9 | cli          | verdict     | execution | skill-flow-a |
| b1 | source skill B         | syncs_to | runtime skill B| install-b.sh:10             |        0.9 | source       | runtime     | deploy    | skill-flow-b |
| b2 | runtime skill B        | exposes  | command B      | install-b.sh:15             |        0.9 | runtime      | cli         | deploy    | skill-flow-b |
| b3 | command B              | returns  | verdict B      | command-b:1                 |        0.9 | cli          | verdict     | execution | skill-flow-b |

## candidate_triads

| id | subject                | relation | object         | evidence         | confidence | subject_role | object_role | route     | group                |
|----|------------------------|----------|----------------|------------------|------------|--------------|-------------|-----------|----------------------|
| c1 | source skill A         | syncs_to | runtime skill A| candidate_answer |        0.9 | source       | runtime     | deploy    | candidate-skill-flow |
| c2 | runtime skill B        | exposes  | command B      | candidate_answer |        0.9 | runtime      | cli         | deploy    | candidate-skill-flow |
| c3 | command B              | returns  | verdict B      | candidate_answer |        0.9 | cli          | verdict     | execution | candidate-skill-flow |
