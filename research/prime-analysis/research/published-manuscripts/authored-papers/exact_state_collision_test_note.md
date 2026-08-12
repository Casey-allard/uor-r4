# Exact-State Collision Test

For a fixed wheel W_r, the exact target-conditioned discrete state up to that scale
is determined by the residue class n mod W_r (equivalently: the full branch/base/fiber
trajectory through that scale).

Therefore, if a prime and a semiprime occur in the same residue class modulo W_r,
they share the same exact target-conditioned state up to that wheel scale.

This test searches for explicit collisions:
- one prime
- one semiprime with both factors larger than the active wheel prime cutoff
in the same residue class modulo W_r.


## Main implication

If such collisions exist, then the exact recursive state up to that scale is not yet sufficient to certify primality.
