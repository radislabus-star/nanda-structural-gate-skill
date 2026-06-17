# Skill Flow Example

## triads

| id | subject                | relation | object         | evidence                    | confidence | subject_role | object_role | route     | group      |
|----|------------------------|----------|----------------|-----------------------------|------------|--------------|-------------|-----------|------------|
| t1 | source skill           | syncs_to | runtime skill  | scripts/install-local.sh:10 |        0.9 | source       | runtime     | deploy    | skill-flow |
| t2 | runtime skill          | provides | trigger rule   | SKILL.md:2                  |        0.9 | runtime      | trigger     | agent     | skill-flow |
| t3 | runtime skill          | exposes  | nanda-check    | scripts/install-local.sh:15 |        0.9 | runtime      | cli         | deploy    | skill-flow |
| t4 | nanda-check            | returns  | gate verdict   | nanda-check:1               |        0.9 | cli          | verdict     | execution | skill-flow |

## candidate_triads

| id | subject                | relation | object         | evidence         | confidence | subject_role | object_role | route     | group                |
|----|------------------------|----------|----------------|------------------|------------|--------------|-------------|-----------|----------------------|
| c1 | source skill           | syncs_to | runtime skill  | candidate_answer |        0.9 | source       | runtime     | deploy    | candidate-skill-flow |
| c2 | runtime skill          | provides | trigger rule   | candidate_answer |        0.9 | runtime      | trigger     | agent     | candidate-skill-flow |
| c3 | runtime skill          | exposes  | nanda-check    | candidate_answer |        0.9 | runtime      | cli         | deploy    | candidate-skill-flow |
| c4 | nanda-check            | returns  | gate verdict   | candidate_answer |        0.9 | cli          | verdict     | execution | candidate-skill-flow |
