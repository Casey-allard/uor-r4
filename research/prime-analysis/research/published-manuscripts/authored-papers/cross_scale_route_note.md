# Cross-Scale Identity Tracking

This experiment takes a **single exact coarse route** at wheel scale `W=30030`, identified by one residue class:

- coarse residue mod 30030: `1`

Then it tracks how that exact route splits into finer lift coordinates at:

- `W=510510 = 17 * 30030`, via `x17`
- `W=9699690 = 19 * 510510`, via `(x17, x19)`

## Main finding

A single exact coarse route does **not** stay pure under higher refinement.

Inside one 17x19 lift block (323 higher cells), the same coarse route contains:
- primes
- large-factor semiprimes
- other composites

So the exact recursive routing geometry is real, but a coarse route does not by itself certify primality.
It recursively partitions the search space into finer subroutes.

This is the exact picture behind the router analogy:
- low scale = coarse bucket
- higher scale = recursive refinement into subroutes
- useful compression may still exist locally, but the finite coarse state is not a final oracle
