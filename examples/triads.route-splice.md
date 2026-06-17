# Route Splice Example

## triads

| id | subject   | relation | object      | evidence     | confidence | subject_role | object_role | route    | group  |
|----|-----------|----------|-------------|--------------|------------|--------------|-------------|----------|--------|
| a1 | Alpha Ltd | supplies | Equipment A | deal-a.md:10 |        0.9 | supplier     | goods       | delivery | deal-a |
| a2 | Beta LLC  | pays     | Alpha Ltd   | deal-a.md:20 |        0.9 | buyer        | supplier    | payment  | deal-a |
| b1 | Gamma Ltd | supplies | Equipment B | deal-b.md:10 |        0.9 | supplier     | goods       | delivery | deal-b |
| b2 | Delta LLC | pays     | Gamma Ltd   | deal-b.md:20 |        0.9 | buyer        | supplier    | payment  | deal-b |

## candidate_triads

| id | subject   | relation | object      | evidence         | confidence | subject_role | object_role | route    | group          |
|----|-----------|----------|-------------|------------------|------------|--------------|-------------|----------|----------------|
| c1 | Alpha Ltd | supplies | Equipment A | candidate_answer |        0.9 | supplier     | goods       | delivery | candidate-deal |
| c2 | Delta LLC | pays     | Gamma Ltd   | candidate_answer |        0.9 | buyer        | supplier    | payment  | candidate-deal |
