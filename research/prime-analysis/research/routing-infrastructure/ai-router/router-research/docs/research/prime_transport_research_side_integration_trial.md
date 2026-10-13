# Prime Transport Research-Side Integration Trial

## Purpose

This note records the smallest research-side integration trial for the current
prime-transport routing policy, executed through the already-established
research-only wrapper boundary and bounded trace family.

The fixed comparison remained:

1. `static_only`
2. `gap_only`
3. `base_gap + fallback`

No live router seam was touched and no new integration boundary was introduced.

## Trial Result

The trial reproduced the established guarded benchmark result on the same
bounded traces.

Aggregate metrics:

### `static_only`

- unique route keys: `2233.0`
- unique emitted route keys: `2233.0`
- route reuse fraction: `0.0`
- promotion route fraction: `0.0`
- promotion step fraction: `0.0`
- effective resolved fraction: `1.0`
- mean unresolved among nonpromoted routes: `0.0`
- route decision instability: `0.0`

### `gap_only`

- unique route keys: `75.5`
- unique emitted route keys: `280.0`
- route reuse fraction: `0.9153846153846155`
- promotion route fraction: `0.6687298741326874`
- promotion step fraction: `0.9532467532467532`
- effective resolved fraction: `0.9960039960039964`
- mean unresolved among nonpromoted routes: `0.026601364321952547`
- route decision instability: `0.0`

### `base_gap`

- unique route keys: `169.0`
- unique emitted route keys: `301.0`
- route reuse fraction: `0.8151848151848151`
- promotion route fraction: `0.2167907268170426`
- promotion step fraction: `0.45274725274725275`
- effective resolved fraction: `0.9813186813186814`
- mean unresolved among nonpromoted routes: `0.014803465594151983`
- route decision instability: `0.0`

## Reading

`base_gap` remains stable under the trial:

- route decision instability stays at `0.0`
- the policy still occupies the intended middle position between the two
  controls
- the benchmark-facing behavior matches the prior guarded benchmark result,
  with no drift in the aggregate numbers

So the current result is:

- `static_only` is still too exact to be a reusable routing policy
- `gap_only` is still too fallback-heavy
- `base_gap` remains the balanced policy under the guarded research-side trial

## Conclusion

The policy is now ready for a larger **research-side** benchmark experiment,
provided the same guarded constraints remain in place:

- no live seam changes
- no runtime router replacement
- bounded trace family first, or only a very small extension of it
- explicit fallback accounting on every run

The main remaining risk is unchanged:

- fallback burden, not instability

So the next safe step is a larger research-side benchmark experiment through
the same wrapper boundary, not live integration.
