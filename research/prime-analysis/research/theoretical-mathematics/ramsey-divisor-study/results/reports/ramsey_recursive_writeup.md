# Ramsey Recursive Exploratory Study

## Scope
- Families: fibonacci_block_growth, lucas_like_growth, balanced_binary_partitions, prime_anchor_partitioning, random_baseline
- Sizes: [8, 10, 12, 14, 16, 18, 20, 22, 24]
- Seeds: [0, 1, 2, 3]

## Metric definitions
- `max_mono_clique_present`: max of largest red clique and largest blue clique.
- `max_mono_independent_present`: max monochromatic independent set size (dual of clique in 2-color complete graphs).
- `delay_score = n / max_mono_clique_present` (higher is better).
- `force_index = max_mono_clique_present / n` (lower is better).
- Spectral metrics are from red adjacency matrix eigenvalues (cheap, exact eigensolve).

## Family ranking (by mean delay score)
- fibonacci_block_growth: mean_delay=3.277, mean_force=0.316, mean_max_clique=4.778
- random_baseline: mean_delay=3.131, mean_force=0.339, mean_max_clique=5.028
- balanced_binary_partitions: mean_delay=3.009, mean_force=0.344, mean_max_clique=5.333
- lucas_like_growth: mean_delay=2.976, mean_force=0.347, mean_max_clique=5.278
- prime_anchor_partitioning: mean_delay=2.185, mean_force=0.468, mean_max_clique=7.222

## Signature notes
- fibonacci_block_growth: red-edge density varies (0.511..0.607).
- fibonacci_block_growth: mean signature distance to phi is 0.0131.
- lucas_like_growth: red-edge density varies (0.444..0.607).
- lucas_like_growth: mean signature distance to phi is 0.0561.
- balanced_binary_partitions: red-edge density varies (0.286..0.714).
- balanced_binary_partitions: mean signature distance to phi is 0.6180.
- prime_anchor_partitioning: red-edge density varies (0.208..0.636).
- prime_anchor_partitioning: mean signature distance to phi is 0.6004.
- random_baseline: red-edge density varies (0.357..0.607).

## Recommendation
- Decision: **KEEP**
- Rationale: At least one structured family outperforms random baseline on both delay score and force index.
- Best observed family in this screen run: **fibonacci_block_growth** (exploratory only; not a theorem).

## Honesty / limits
- This is a small finite-n exploratory screen and does not establish asymptotic Ramsey bounds.
- Family definitions here are heuristic constructions; different recursion rules could change outcomes.
- Independent-set and clique metrics are tightly coupled in 2-color complete graphs.
