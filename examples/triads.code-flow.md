# Code Flow Example

## triads

| id | subject              | relation    | object               | evidence             | confidence | subject_role | object_role | route     | group               |
|----|----------------------|-------------|----------------------|----------------------|------------|--------------|-------------|-----------|---------------------|
| t1 | source skill         | installs_to | runtime skill        | install-local.sh:10  |        0.9 | source       | runtime     | deploy    | source-runtime-flow |
| t2 | runtime skill        | exposes     | nanda-check          | install-local.sh:15  |        0.9 | runtime      | cli         | deploy    | source-runtime-flow |
| t3 | runtime skill        | exposes     | nanda-gate           | install-local.sh:16  |        0.9 | runtime      | cli         | deploy    | source-runtime-flow |
| t4 | nanda-gate           | calls       | nanda-check          | nanda-gate:5         |        0.9 | cli          | checker     | execution | source-runtime-flow |

## candidate_triads

| id | subject              | relation    | object               | evidence         | confidence | subject_role | object_role | route     | group               |
|----|----------------------|-------------|----------------------|------------------|------------|--------------|-------------|-----------|---------------------|
| c1 | source skill         | installs_to | runtime skill        | candidate_answer |        0.9 | source       | runtime     | deploy    | candidate-code-flow |
| c2 | runtime skill        | exposes     | nanda-check          | candidate_answer |        0.9 | runtime      | cli         | deploy    | candidate-code-flow |
| c3 | runtime skill        | exposes     | nanda-gate           | candidate_answer |        0.9 | runtime      | cli         | deploy    | candidate-code-flow |
| c4 | nanda-gate           | calls       | nanda-check          | candidate_answer |        0.9 | cli          | checker     | execution | candidate-code-flow |
