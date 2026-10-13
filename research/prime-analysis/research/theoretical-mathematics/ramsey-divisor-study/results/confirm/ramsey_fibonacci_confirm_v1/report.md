# Confirm Pass: Fibonacci Ramsey Signal (v1)

## Setup
- task_id: `ramsey_fibonacci_confirm_v1`
- families: fibonacci base, random baseline, balanced-binary control, and 3 nearby Fibonacci perturbations
- sizes: [8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28]
- seeds: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] (count=16)

## Family means
- fibonacci_mirrored_growth: delay=3.5765, force=0.2907, max_clique=4.892
- fibonacci_block_growth: delay=3.5125, force=0.2971, max_clique=4.983
- fibonacci_phi_partition: delay=3.4216, force=0.3080, max_clique=5.119
- random_baseline: delay=3.3030, force=0.3215, max_clique=5.318
- fibonacci_offbyone_growth: delay=3.1073, force=0.3369, max_clique=5.665
- balanced_binary_partitions: delay=3.0758, force=0.3357, max_clique=5.818

## Robustness checks (paired by n,seed vs Fibonacci base)
- fibonacci_block_growth - random_baseline: mean_delay_diff=0.2095 [0.1531, 0.2659], mean_force_diff=-0.0244 [-0.0314, -0.0173], delay_win_rate=0.358, force_win_rate=0.358
- fibonacci_block_growth - balanced_binary_partitions: mean_delay_diff=0.4367 [0.3326, 0.5409], mean_force_diff=-0.0386 [-0.0477, -0.0294], delay_win_rate=0.591, force_win_rate=0.591
- fibonacci_block_growth - fibonacci_mirrored_growth: mean_delay_diff=-0.0640 [-0.1177, -0.0103], mean_force_diff=0.0064 [0.0016, 0.0112], delay_win_rate=0.085, force_win_rate=0.085
- fibonacci_block_growth - fibonacci_phi_partition: mean_delay_diff=0.0909 [0.0377, 0.1442], mean_force_diff=-0.0108 [-0.0168, -0.0048], delay_win_rate=0.222, force_win_rate=0.222
- fibonacci_block_growth - fibonacci_offbyone_growth: mean_delay_diff=0.4052 [0.3406, 0.4697], mean_force_diff=-0.0397 [-0.0468, -0.0326], delay_win_rate=0.574, force_win_rate=0.574

## Direct answers
- Does Fibonacci still beat random? Yes.
- Does it beat strongest non-Fibonacci structured control? Yes.
- Do nearby perturbations collapse the edge or preserve it? Mixed; see paired differences above.

## Verdict
- Decision: **REFINE**
- Rationale: Signal persists, but nearby perturbations achieve similar performance; effect may be broader recursive-ratio structure.

## Limits
- Finite small-n confirm pass only; this does not establish asymptotic Ramsey behavior.
- Construction choices are heuristic and could be sensitive to exact recursive assignment rules.
