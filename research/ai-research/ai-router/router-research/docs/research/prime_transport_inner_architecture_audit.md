# Prime Transport Inner Architecture Audit

## Scope

This audit checks the inner representation actually used by the
`geometry_native_sequence_model*` line, rather than the later wrapper and field
selection layers.

It compares:

- the intended exact-layer and routing abstraction from
  `prime_transport_exact_layer_consolidated.md` and
  `prime_transport_routing_abstraction.md`
- the stated geometry-native contract from the sequence-model notes
- the actual implementation in
  `tools/prime_transport/geometry_native_sequence_model_v3.py`,
  `tools/prime_transport/geometry_native_sequence_model_v7.py`,
  `tools/prime_transport/geometry_native_sequence_model_v11.py`,
  `tools/prime_transport/geometry_native_sequence_model_v13.py`,
  `tools/prime_transport/geometry_native_sequence_model_v14.py`,
  `tools/prime_transport/geometry_native_sequence_model_v21.py`,
  and the later v29-v33 field files

## Bottom Line

The recent failures are more likely due to an under-specified inner
representation than to the wrapper/search branch alone.

The current sequence-model line is not implementing the intended inner
bundle/orbit/transport architecture in a native way. It is mostly:

1. a manually updated discrete snapshot state
2. flattened into one-hot feature vectors
3. passed through a small MLP readout
4. with outer wrapper logic choosing among alternate snapshot rewrites

That is a workable bounded benchmark apparatus, but it is not the same thing as
bundle-native computation on orbital/spin coordinates.

## What Is Actually Present

### 1. Exact discrete chart counters are present

The implementation does maintain a small exact-like chart state:

- `b, phi, r` from `_initial_geometry_state`
- `next_return_gap` as a residual routing tag

This is real and explicit in
`tools/prime_transport/geometry_native_sequence_model_v3.py:78-79` and updated
in `:164-167`.

But the update law is not an orbit-native transport object. It is a token-local
table lookup over fixed deltas:

- `TOKEN_DB_V3`, `TOKEN_DPHI_V3`, `TOKEN_DR_V3`, `TOKEN_DGAP_V3`
  in `:46-49`
- then modular counter updates in `step_discourse_world` in `:164-167`

So the implementation has discrete chart counters, but not a deeper bundle or
orbit transport representation.

### 2. Explicit bounded discourse memory is present

The line does natively represent:

- `focus`
- `speaker`
- `topic`
- `style`
- per-entity `tags`

This is the true inner working memory in practice, alongside the chart
counters. It is updated directly in `step_discourse_world`
`tools/prime_transport/geometry_native_sequence_model_v3.py:101-172`.

### 3. The actual computation is mostly feature encoding plus MLP readout

The operative representation used for learning is `_build_feature_vector_v3`
in `tools/prime_transport/geometry_native_sequence_model_v3.py:225-241`.

That feature vector is:

- token one-hot
- one-hot of `b`
- one-hot of `phi`
- one-hot of `r`
- one-hot of `next_return_gap`
- one-hot of `focus`, `speaker`, `topic`, `style`
- one-hot of `referent_role`, `referent_entity`
- one-hot tags

Then the "geometry-native model" is only a feed-forward readout:

- `GeometryReadoutV3` in `:284-296`
- `GeometryNativeSequenceModelV3` in `:299-349`

So the inner computation is not a bundle/orbit engine. It is a structured
symbolic feature encoder plus MLP.

## What Is Only Approximated By Outer Wrapper Logic

### 4. Divisibility / semiprime transport is not native state

The divisibility bridge in v11-v13 is real as a bounded mechanism, but it is
not represented as persistent computational state.

What the code actually does:

- compute prime-coded deltas from the current snapshot
- form `bridge_semiprime = proxy_prime * bridge_prime`
- decode a bridged delta by divisibility tests
- overwrite only `referent_role` and `referent_entity`

See:

- `tools/prime_transport/geometry_native_sequence_model_v11.py:52-132`
- `tools/prime_transport/geometry_native_sequence_model_v12.py:242-304`
- `tools/prime_transport/geometry_native_sequence_model_v13.py:50-108`

What it does **not** do:

- store semiprime/composite transport objects in state
- transport prime factors across steps as native coordinates
- make divisibility hubs part of the evolving latent state

So composite / semiprime / divisibility transport is only indirect and
ephemeral. It is a feature rewrite layer, not native transport state.

### 5. Orbital/spin charts are not used as computational coordinates

The exact-layer notes say the stronger predictive object is:

- `spin_H(j)` and `(b, spin_H)` in
  `prime_transport_exact_layer_consolidated.md:30-36`

and the routing note says the best current abstraction is:

- `R_min = (b, phi, r, next_return_gap)` with `spin_H` as the stronger full
  predictive state in
  `prime_transport_routing_abstraction.md:12-13` and `:147-154`

But in the sequence-model implementation:

- there is no `spin_H`
- there is no admissibility word
- there is no orbit index `j`
- there is no return-grammar object beyond `next_return_gap`
- there are no imports from `hyperbolic_router_so8.py`, `hopf_routing_demo.py`,
  or router-side bundle/chart code at all

The `rg` search over `tools/prime_transport/geometry_native_sequence_model_v*.py`
found no use of:

- `spin`
- `orbit`
- `bundle`
- `Hopf`
- `admissib`

So orbital/spin charts are not functioning as true computational coordinates in
this line. At most, the discrete counters act as labels that are flattened into
features.

### 6. Later wrappers mostly choose among alternate snapshot rewrites

The adaptive line from v13 onward mainly operates by selecting among alternate
feature views:

- `_bridged_snapshot` in v13 rewrites snapshot fields
- `_variant_features` in v14 materializes a whole feature tensor per variant
- v21-v33 summarize regions and choose among cached variants and regional
  decompositions

See:

- `tools/prime_transport/geometry_native_sequence_model_v13.py:50-108`
- `tools/prime_transport/geometry_native_sequence_model_v14.py:45-56`
- `tools/prime_transport/geometry_native_sequence_model_v21.py:53-166`
- `tools/prime_transport/geometry_native_sequence_model_v29.py:59-108`
- `tools/prime_transport/geometry_native_sequence_model_v30.py:77-168`
- `tools/prime_transport/geometry_native_sequence_model_v33.py:84-259`

So most later progress is wrapper logic over featureized snapshots, not richer
native inner coordinates.

## Direct Answers

### What parts of the original intended architecture are actually present?

Present:

- discrete phase-fiber-scale counters `b, phi, r`
- `next_return_gap` as a small dynamic residual tag
- explicit bounded discourse/memory state
- deterministic token-conditioned state updates
- bounded prime/semiprime bridge rewrites of referent fields

### What parts are only approximated by outer wrapper logic?

Only approximated:

- adaptive chart selection
- regional regime selection
- divisibility-mediated realignment as persistent transport
- any broader adaptive geometry beyond the immediate snapshot

These are implemented as feature-view selection, not inner-state redesign.

### Is composite/semiprime/divisibility transport represented natively in the state, or only indirectly?

Only indirectly.

It appears only as temporary arithmetic used to rewrite `referent_role` /
`referent_entity`. It is not stored as part of the state and is not transported
step to step.

### Are orbital/spin charts acting as true computational coordinates, or mostly as labels/features?

Mostly absent; where anything comparable exists, it is acting as
labels/features.

The actual learned path consumes one-hot snapshot features, not orbit-native or
spin-native coordinates.

### Are admissibility filters actually constraining sequence evolution in the way originally intended?

No.

The sequence-model line does not implement admissibility filtering as a native
constraint on evolution. Tokens are sampled from fixed distributions in
`generate_dataset_v3` and transfer generators, then the state is updated
deterministically. `next_return_gap` is tracked, but there is no actual
admissibility filter or orbit-validity gate constraining transitions.

### What is the single most likely representational gap?

The single most likely gap is:

- the intended orbit-native predictive state is missing

The code uses:

- explicit snapshot labels + MLP

when the intended architecture increasingly points toward:

- native transport on an exact chart plus a compact predictive residual
- most plausibly `R_min = (b, phi, r, next_return_gap)` enriched by a true
  return-grammar / spin-like transport object

In other words, the implementation never rebuilt the inner representation
around orbit-native predictive transport. It kept wrapping a snapshot encoder.

## Recommendation

Recommend exactly one next branch:

- rebuild part of the inner representation

Specifically, do **not** continue the wrapper/search branch as the main line.
The cleaner next step is to rebuild the inner state around a native
orbit/predictive transport object rather than another outer selector on top of
snapshot features.
