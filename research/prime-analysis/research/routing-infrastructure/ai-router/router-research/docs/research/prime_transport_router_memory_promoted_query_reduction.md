# Prime Transport Router Memory Promoted-Query Reduction

## Purpose

This note records the first bounded attempt to reduce promoted-query burden in
the router-memory task loop without changing the overall memory architecture.

The question was narrow:

- are promoted queries concentrated enough that one cheap read-side refinement
  can help?
- if so, can that refinement lower promoted-query burden without materially
  harming carried-state retrieval, route reuse, or stability?

## Concentration Profile

Using the existing task-loop workload, the promoted-query burden is **not**
diffuse.

It is concentrated mainly by lift family:

- baseline promoted-query total: `15580`
- share coming from the deeper `30030 -> 510510` family: `0.9701540436456996`

But within that deeper family, the burden is **not** concentrated tightly into
one tiny hot class:

- top `phi0` share: `0.1058408215661104`
- top gap share: `0.05044929396662388`
- top single route share: `0.007958921694480103`

So the honest reading is:

- the burden is concentrated enough to justify trying one cheap refinement
- but it is still spread across many repeated hard cases inside the deeper
  family

## Refinement Candidate

The single tested refinement was:

- `phi0_hint_refined`

Meaning:

- keep the same stored memory state
- keep the same route classes
- add only one tiny read-side hint derived from the leading phase coordinate
  `phi0`

This hint is smaller than full richer retrieval and stays within the current
router-memory architecture.

## Comparison

### Baseline

- route reuse fraction: `0.8151848151848151`
- query hit fraction on reuse: `1.0`
- task retrieval accuracy: `0.9300522166708776`
- promoted-query fraction on reuse: `0.48887856062924245`
- promotion step fraction: `0.45274725274725275`
- effective task resolution fraction: `0.7386524819618124`
- route decision instability: `0.0`

### `phi0_hint_refined`

- route reuse fraction: `0.8151848151848151`
- query hit fraction on reuse: `1.0`
- task retrieval accuracy: `0.6832439966572033`
- promoted-query fraction on reuse: `0.21055889917899204`
- hint-refined fraction on reuse: `0.27831966145025044`
- promotion step fraction: `0.45274725274725275`
- effective task resolution fraction: `0.6152483719549754`
- route decision instability: `0.0`

## Reading

### Is promoted-query burden concentrated enough for one cheap refinement to help?

Yes, but only partially.

The burden is concentrated strongly enough in the deeper lift family that one
cheap refinement can reduce promoted queries materially.

### Does the refinement reduce promoted querying materially?

Yes.

The tested hint reduces promoted-query fraction on reuse from:

- `0.48887856062924245`

to:

- `0.21055889917899204`

So the burden drop itself is real.

### Does it preserve the stable carried-state behavior?

Only partly, and not well enough.

What it preserves:

- route reuse
- query-hit rate
- zero instability

What it breaks too much:

- task retrieval accuracy falls from `0.9301` to `0.6832`
- effective task resolution falls from `0.7387` to `0.6152`

So the hint lowers promoted-query burden by collapsing too many distinct memory
cases together.

## Conclusion

The promoted-query burden is structured enough that a cheap read-side hint can
lower it, but not structured enough that the tested one-key refinement is a
good trade.

So the bounded architectural conclusion is:

- do **not** replace the current router-memory read path with the tested
  `phi0` hint refinement
- keep the current memory layer unchanged
- treat the remaining promoted-query burden as a real deeper-family cost, not
  as a problem already solved by a tiny read-side split

That means the memory-layer architecture remains viable, but the next useful
reduction in promoted-query burden will likely need something richer than one
cheap phase-side hint.
