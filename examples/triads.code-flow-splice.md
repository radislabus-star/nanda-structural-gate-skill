# Code Flow Splice Example

## triads

| id | subject              | relation    | object               | evidence             | confidence | subject_role | object_role | route     | group          |
|----|----------------------|-------------|----------------------|----------------------|------------|--------------|-------------|-----------|----------------|
| a1 | source skill A       | installs_to | runtime skill A      | install-a.sh:10      |        0.9 | source       | runtime     | deploy    | flow-a         |
| a2 | runtime skill A      | exposes     | command A            | install-a.sh:15      |        0.9 | runtime      | cli         | deploy    | flow-a         |
| a3 | command A            | calls       | checker A            | command-a:5          |        0.9 | cli          | checker     | execution | flow-a         |
| b1 | source skill B       | installs_to | runtime skill B      | install-b.sh:10      |        0.9 | source       | runtime     | deploy    | flow-b         |
| b2 | runtime skill B      | exposes     | command B            | install-b.sh:15      |        0.9 | runtime      | cli         | deploy    | flow-b         |
| b3 | command B            | calls       | checker B            | command-b:5          |        0.9 | cli          | checker     | execution | flow-b         |

## candidate_triads

| id | subject              | relation    | object               | evidence         | confidence | subject_role | object_role | route     | group               |
|----|----------------------|-------------|----------------------|------------------|------------|--------------|-------------|-----------|---------------------|
| c1 | source skill A       | installs_to | runtime skill A      | candidate_answer |        0.9 | source       | runtime     | deploy    | candidate-code-flow |
| c2 | runtime skill B      | exposes     | command B            | candidate_answer |        0.9 | runtime      | cli         | deploy    | candidate-code-flow |
| c3 | command B            | calls       | checker B            | candidate_answer |        0.9 | cli          | checker     | execution | candidate-code-flow |
