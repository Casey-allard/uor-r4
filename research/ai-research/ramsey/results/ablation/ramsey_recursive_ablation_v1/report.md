# Ramsey Recursive Ablation v1

## Setup
- task_id: `ramsey_recursive_ablation_v1`
- base family: `fibonacci_block_growth_base`
- controls: `fibonacci_mirrored_growth`, `balanced_binary_partitions`, `random_baseline`
- sizes: [8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28]
- seeds: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23] (count=24)

## Family means
- fibonacci_mirrored_growth: delay=3.5894, force=0.2898, max_clique=4.879
- fib_shallow_stop_expand: delay=3.5029, force=0.2985, max_clique=5.004
- fibonacci_block_growth_base: delay=3.4834, force=0.2997, max_clique=5.030
- random_baseline: delay=3.2902, force=0.3230, max_clique=5.341
- fib_de_fibonacci_plus1: delay=3.2534, force=0.3217, max_clique=5.394
- fib_linear_hash: delay=3.2516, force=0.3184, max_clique=5.390
- fib_binary_growth_schedule: delay=3.2184, force=0.3243, max_clique=5.466
- balanced_binary_partitions: delay=3.0758, force=0.3357, max_clique=5.818
- fib_parity_hash: delay=2.8757, force=0.3565, max_clique=6.091
- fib_no_parity_flip: delay=2.8503, force=0.3595, max_clique=6.186
- fib_asymmetry_off: delay=2.0244, force=0.5053, max_clique=8.742

## One-knob ablation impacts (paired vs base)
- fibonacci_block_growth_base - balanced_binary_partitions: delay_diff=0.4076 [0.3219, 0.4934], force_diff=-0.0360 [-0.0437, -0.0284]
- fibonacci_block_growth_base - fib_asymmetry_off: delay_diff=1.4589 [1.3923, 1.5256], force_diff=-0.2056 [-0.2127, -0.1986]
- fibonacci_block_growth_base - fib_binary_growth_schedule: delay_diff=0.2650 [0.2114, 0.3186], force_diff=-0.0246 [-0.0300, -0.0192]
- fibonacci_block_growth_base - fib_de_fibonacci_plus1: delay_diff=0.2300 [0.1797, 0.2802], force_diff=-0.0221 [-0.0273, -0.0168]
- fibonacci_block_growth_base - fib_linear_hash: delay_diff=0.2318 [0.1816, 0.2820], force_diff=-0.0187 [-0.0237, -0.0138]
- fibonacci_block_growth_base - fib_no_parity_flip: delay_diff=0.6330 [0.5748, 0.6912], force_diff=-0.0599 [-0.0653, -0.0544]
- fibonacci_block_growth_base - fib_parity_hash: delay_diff=0.6077 [0.5556, 0.6597], force_diff=-0.0568 [-0.0615, -0.0521]
- fibonacci_block_growth_base - fib_shallow_stop_expand: delay_diff=-0.0196 [-0.0556, 0.0164], force_diff=0.0012 [-0.0028, 0.0051]
- fibonacci_block_growth_base - fibonacci_mirrored_growth: delay_diff=-0.1060 [-0.1526, -0.0594], force_diff=0.0099 [0.0056, 0.0142]
- fibonacci_block_growth_base - random_baseline: delay_diff=0.1931 [0.1451, 0.2412], force_diff=-0.0234 [-0.0293, -0.0174]

## Driver interpretation
- Recursive asymmetry contributes, but ordering and assignment interactions are substantial.
- Mirrored ordering outperforming base indicates edge is not tied to one canonical Fibonacci ordering.
- Hash/parity/growth changes alter outcomes materially, suggesting interaction rather than single-knob causality.

## Verdict
- Decision: **REFINE**
- Rationale: No single dominant Fibonacci-specific knob. Mirror ordering improves performance, while hash/parity/growth knobs still matter; effect appears interaction-driven.

## Limits
- Finite small-n ablation only; no asymptotic claim.
- Knob factorization is near-orthogonal but not perfectly orthogonal.
