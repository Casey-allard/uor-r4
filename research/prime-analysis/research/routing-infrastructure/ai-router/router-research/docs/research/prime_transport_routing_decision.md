# Prime Transport Routing Decision

## Decision Scope

This note freezes the current guarded research-side routing decision.

It is based on:

- the guarded benchmark validation of `base_gap + fallback`
- the direct head-to-head against the canonical angular Hopf baseline
- the transfer comparison in which delayed refinement, fallback accounting,
  and predictive partitioning were added back into the unchanged angular Hopf
  path

No live MUDBench seam work is implied by this note.

## What Was Compared

Three practically relevant policies were compared on the guarded
research-side benchmark boundary:

1. raw angular Hopf baseline
2. prime-transport `base_gap + fallback`
3. guarded angular Hopf with transferred routing discipline

The fixed prime-transport policy was:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

## What Was Learned

### 1. Raw angular Hopf lost partly because it lacked routing discipline

The first direct head-to-head showed that the canonical angular Hopf baseline
was extremely compact but far too coarse as a practical routing policy on the
guarded trace family:

- very high reuse
- zero fallback
- but effective resolved fraction only `0.1681`

So part of the angular loss was procedural:

- unresolved route classes were not being handled explicitly

### 2. Guarded angular recovers resolution only by promoting almost everything

When the three transferred ideas were added back into the unchanged angular
path:

- delayed refinement
- explicit fallback accounting
- predictive partitioning

the angular route recovered predictive resolution, but only by collapsing into
near-total fallback:

- guarded angular fallback step fraction: `0.9883`
- guarded angular effective resolved fraction: `1.0`

This means routing discipline repairs the angular failure mode, but not
cheaply.

### 3. Prime transport remains better on the key tradeoff

`base_gap + fallback` continues to reach nearly the same resolution with much
lower fallback burden:

- `base_gap` effective resolved fraction: `0.9900`
- `base_gap` fallback step fraction: `0.3312`
- route decision instability: `0.0`

So the remaining advantage of the prime-transport policy survives the
discipline transfer and now looks structural rather than merely procedural.

## Decision

`base_gap + fallback` is now the leading research-side routing candidate.

The canonical angular Hopf path should now be treated as:

- a practical comparison baseline
- a source of compact static routing ideas
- but not the primary guarded research-side policy path

This does **not** mean the angular path is uninformative. It means the current
guarded evidence favors the prime-transport policy as the stronger bounded
policy under directly comparable evaluation.

## Next Action

The next step should be:

- the first guarded research-side benchmark integration trial with
  prime-transport `base_gap + fallback` as the primary policy under test
- angular Hopf retained as the comparison baseline
- the same guarded wrapper boundary
- the same explicit fallback accounting
- still no live MUDBench seam changes

## Frozen Conclusion

The current research-side routing conclusion is:

- routing discipline matters and should be kept
- but discipline alone does not erase the prime-transport advantage
- the leading primary candidate going forward is
  `base_gap = (b, r, next_return_gap)` with fallback to `R_full = (b, spin_H)`
- angular Hopf remains the baseline comparator, not the primary path
