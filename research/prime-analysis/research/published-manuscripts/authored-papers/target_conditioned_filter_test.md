# Target-Conditioned Chained-Filter Test

This test evaluates the exact local-filter product state for specific integers `n` across growing prime filters.

For each active prime cutoff `p_max`, the state is:

- local filter bits: whether `n mod p != 0` for each prime `p <= p_max`
- alive flag: product of those bits

## Main finding

This state detects compositeness **exactly when a prime factor enters the filter chain**.

It does **not** distinguish:
- a prime with all factors greater than 23
- from a composite whose prime factors are all greater than 23

through the tested wheels.

Examples:
- `887` (prime) and `899 = 29*31` have identical chained-filter states through prime 23.
- `1000003` (prime) and `1022117 = 1009*1013` have identical chained-filter states through prime 23.

So the exact chained admissibility product is real, but by itself it behaves like structured factor elimination:
it detects a composite when one of its factors is finally included.
