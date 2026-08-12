# Prime Transport Attractor Identity Experiment

## Minimal Candidate

The single tested attractor-identity candidate is:

- `control_anchor`

It is a bounded converged-neighborhood label derived only from the existing
packet surface:

- packet intent
- local progress stage `(goal, world, combat, pressure)`
- dependency count
- shared task status booleans
- shared world status booleans

So it is:

- smaller than full branch identity
- smaller than full spin identity
- not a new packet field
- not a routing or memory change

Operationally, it is used only as a controller-side cache key for reusing a
previous converged shared neighborhood.

## Aggregate Comparison

- improved packet controller:
  - action correctness: `0.7676989676989677`
  - joint control-loop correctness: `0.49339896262483907`
  - promoted-query fraction on reuse: `0.48843742442001076`
  - route reuse: `0.8411588411588411`
  - local retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - instability: `0.0`
- attractor-identity controller:
  - action correctness: `0.8437895437895437`
  - joint control-loop correctness: `0.6188811188811189`
  - promoted-query fraction on reuse: `0.48843742442001076`
  - route reuse: `0.8411588411588411`
  - local retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - instability: `0.0`

## Reading

### Does the attractor-identity candidate reduce promoted-query burden materially?

No.

On the current bounded experiment:

- promoted-query fraction on reuse stays exactly
  `0.48843742442001076`
- promotion step fraction stays exactly `0.4595404595404595`

So this candidate does **not** solve the main explicit efficiency cost.

### Does it preserve retrieval quality and controller gains?

Yes.

It preserves all the already-validated substrate behavior:

- local retrieval accuracy remains `1.0`
- shared-ledger retrieval accuracy remains `1.0`
- route reuse remains `0.8411588411588411`
- instability remains `0.0`

And it improves control quality materially over the improved packet controller:

- action correctness rises from `0.7676989676989677` to `0.8437895437895437`
- joint control-loop correctness rises from `0.49339896262483907` to
  `0.6188811188811189`
- effective joint-state resolution rises from `0.7603114941566694` to
  `0.8006260406260406`

### Does it look like a real convergence-identity mechanism, or just another lossy local hint?

It looks like a real bounded convergence-identity mechanism for control, but
not yet a burden-reduction mechanism.

The key reason is:

- retrieval quality does not degrade
- route reuse does not degrade
- promoted-query burden does not move
- yet control quality improves substantially

That is not the profile of the rejected `phi0` hint, which reduced burden only
by collapsing distinctions and damaging retrieval quality.

So the current candidate behaves as:

- a useful controller-side converged-neighborhood anchor

not as:

- a lossy surrogate for richer memory identity

## Conclusion

The tested attractor identity does **not** reduce promoted-query burden on the
current unchanged stack.

But it does improve bounded control quality materially while preserving exact
retrieval and zero instability.

So the honest branch-level reading is:

- this is a promising convergence-identity mechanism for control
- it is not yet a solution to promoted-query cost
- promoted querying remains the main explicit efficiency burden

The next bounded step, if desired, should treat attractor identity as a
controller-quality aid rather than as a validated memory-cost reduction.
