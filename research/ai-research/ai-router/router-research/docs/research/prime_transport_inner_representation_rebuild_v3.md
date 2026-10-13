# Prime Transport Inner Representation Rebuild V3

## Purpose

Replace the remaining hand-built `propose semiprime -> project into admissible
set` transition from `r1`/`r2` with a native composite/chart transport
operator.

## Mechanism

`r3` keeps the same bounded native state and the same structured readout family
from `r2`, but changes the transition itself.

The new transition rule is factor-native:

- each semiprime is decoded into an ordered prime-factor pair
- one factor is retained as an anchor from the previous state
- the partner factor is transported directly in factor space using:
  - chart state `(b, phi, r, next_return_gap)`
  - discourse state `(focus, speaker, topic, style, tags)`
  - spin state `spin_bits`
  - token-local transport increments
  - cross-coupling from the opposite composite state
- the next semiprime is then recomposed directly from the transported factor
  pair

There is no proposal-and-fix stage. Valid semiprimes are produced directly by
construction from admissible factor pairs.

## Transition Rule

At each step:

1. decode:
   - `query_semiprime -> (q_left, q_right)`
   - `binding_semiprime -> (b_left, b_right)`

2. choose retained anchor factors from the current composite state using
   chart/discourse parity

3. transport the partner factors by direct factor updates:
   - query partner depends on token transport, style, spin, tag parity, and
     the left binding factor
   - binding partner depends on token transport, chart turn, discourse turn,
     and the left query factor

4. recompose the next semiprimes from the transported factor pairs

This keeps every next state inside the semiprime state space by construction.

## Bounded Experiment

Task:
- same bounded v3 discourse/query task used in `r1` and `r2`

Compared models:
- `geometry_native_sequence_model_v3_reference`
- `geometry_native_sequence_model_r1`
- `geometry_native_sequence_model_r2`
- `geometry_native_sequence_model_r3`
- `tiny_transformer_baseline_r3`

## Results

| model | test loss | test accuracy | query accuracy | params | effective state |
|---|---:|---:|---:|---:|---:|
| v3 reference | 0.0077 | 0.9978 | 0.9878 | 5699 | 10 |
| r1 rebuild | 0.4953 | 0.7361 | 0.7009 | 7427 | 16 |
| r2 rebuild | 0.5985 | 0.7426 | 0.7468 | 219 | 16 |
| r3 rebuild | 0.5963 | 0.7424 | 0.7504 | 219 | 16 |
| tiny transformer | 0.7199 | 0.6958 | 0.6707 | 69763 | 64 |

## Is the transition operator native (not projection-based)?

Yes.

## Architecture Checks

### Is the proposal + projection mechanism fully removed?

Yes.

`r3` does not construct a candidate semiprime and then repair it. The next
state is produced directly from retained anchor factors plus transported
partner factors, then recomposed into a semiprime.

### What exact rule defines the transition operator?

For each of `query_semiprime` and `binding_semiprime`:

1. decode the current semiprime into an ordered factor pair
2. retain one anchor factor from the current pair
3. transport the partner factor in prime-index space using token, chart,
   discourse, spin, and cross-composite signals
4. recompose the next semiprime from the retained anchor and transported
   partner

The operator is therefore:
- factor-wise
- chart-aware
- cross-coupled between query and binding composites
- semiprime-valid by construction

### How does composite/factor structure influence transitions?

- the state is updated in factor space, not directly in semiprime label space
- one prime factor from each composite is always retained as the transition
  anchor
- the partner factor is transported using the opposite composite's retained
  factor as part of the update signal
- query tokens add extra cross-coupling by mixing the right-side factors across
  query/binding states

### How do chart/spin coordinates influence transitions?

- chart state `(b, phi, r, next_return_gap)` is collapsed into a chart-turn
  direction used in anchor selection and partner transport
- discourse state `(focus, speaker, topic, style, tags)` contributes a
  discourse-turn and tag-turn signal used in partner transport
- `spin_bits` contribute a spin-turn signal that shifts partner-factor motion

### What guarantees that transitions remain admissible?

Admissibility is guaranteed by retained-factor transport:

- each next semiprime contains one factor copied from the previous semiprime
- therefore `gcd(next_query, previous_query) > 1`
- and `gcd(next_binding, previous_binding) > 1`

On the bounded evaluation sample, the resulting `admissible_transition` flag is
`1.0` on average with min `1` and max `1`, so the operator is behaving as
constructed rather than relying on repair.

## Interpretation

`r3` is a small but real architectural step beyond `r2`.

- overall test accuracy is effectively flat versus `r2`
  - `0.7426 -> 0.7424`
- query accuracy improves slightly
  - `0.7468 -> 0.7504`
- loss improves slightly
  - `0.5985 -> 0.5963`

So `r3` does not recover the old approximate v3 performance, but it does
replace the last major non-native piece in the transition path and modestly
improves query behavior.

## Direct Answers

- Is the proposal + projection mechanism fully removed?
  - yes
- What exact rule defines the transition operator?
  - retained-factor plus transported-partner recomposition in factor space
- How does composite/factor structure influence transitions?
  - state evolves by direct factor updates with cross-composite coupling
- How do chart/spin coordinates influence transitions?
  - they define chart-turn, discourse-turn, tag-turn, and spin-turn offsets in
    partner-factor motion
- What guarantees that transitions remain admissible?
  - one previous factor is always retained in each next semiprime

## What Remains Approximate?

- the factor transport law is still hand-designed rather than learned
- the chart/orbit structure is still coarse and indexed, not a fuller bundle
  geometry
- readout is structured and native, but still table-based rather than an exact
  transport/readout algebra

## Next Smallest Rebuild Step

Learn the factor-space transport law itself over the native composite/chart
state, while keeping the no-flatten and no-projection constraints in place.
