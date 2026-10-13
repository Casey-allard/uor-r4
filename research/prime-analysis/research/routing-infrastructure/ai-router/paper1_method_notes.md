# Paper 1 Method Notes

These notes are paper-supporting technical prose for the canonical Paper 1 method. They are not broad-program framing.

## Routing mechanism

The routing function takes a normalized input vector and assigns it to a fixed sector without learning a gate matrix.

The canonical procedure is:
1. L2-normalize the input vector.
2. Project it onto a fixed 4D subspace using the canonical coordinate tuple `(3, 65, 2, 21)`.
3. Interpret the 4D coordinates through Hopf-style angular coordinates.
4. Discretize those coordinates into a finite routing budget `K`.
5. Use the resulting discrete sector as the routing address.

The important structural feature is that routing is fixed by geometry rather than learned by a separate gating network.

## Why the Hopf map is used

The method needs a low-dimensional angular parameterization that preserves meaningful structure after normalization. The Hopf-style parameterization is used because it splits the 4D projection into coordinates with a natural angular interpretation:
- a coarse base coordinate,
- an angular difference term,
- and a fiber-phase term.

In the canonical Paper 1 framing, the Hopf map is not justified as a grand theory of all routing. It is justified because it is the fixed low-dimensional geometry that survives the repo’s narrowed evidence chain:
- it discriminates structured semantic embeddings from controls,
- it beats the adaptive `K`-means comparator on the proxy,
- and it yields the canonical sublinear routing law on the semantic proxy.

## What the transport term is doing

The transport term modifies the raw fiber-phase coordinate by adding a geometry-dependent correction:

`transported_alpha = alpha + 0.5 * lambda * cos(2 * chi) * delta`

Paper 1 should treat this carefully.

What it does mechanically:
- `alpha` is the raw fiber-phase coordinate,
- `delta` is the angular difference coordinate,
- `chi` is a base-coordinate term derived from the 4D projection,
- `lambda` scales the correction.

What it means operationally:
- the correction perturbs the phase coordinate in a way that depends on where the point sits in the low-dimensional Hopf parameterization,
- so two inputs with similar raw phase can still route differently if their base geometry differs.

What Paper 1 should and should not claim:
- It can describe the transport term because it is part of the canonical routing function in the current manuscript source.
- It should not build the paper around transport as an independent contribution.
- The main paper claim is about fixed geometric routing overall, not about transport-specific novelty.

## Why routing concentrates on the proxy

On the semantic proxy, routing concentration is the core positive result.

The paper-supporting interpretation is:
- the proxy embeddings are semantically structured rather than uniform on the normalized sphere,
- the fixed Hopf-derived sector map partitions that sphere unevenly with respect to the actual data distribution,
- so traffic accumulates in a subset of sectors instead of filling the routing budget uniformly,
- which lowers the effective routing footprint relative to dense routing and relative to weaker controls.

This is why the paper can report a sublinear routing law on the proxy rather than merely a sparse-looking histogram at one `K`.

The controls matter here:
- structured vs column-permuted proxy controls show the effect depends on semantic angular structure,
- Hopf vs adaptive `K`-means shows this is not reducible to generic clustering,
- norm-invariance results support the narrower interpretation that the main effect is angular rather than radial-shell driven.

## Why the geometry-specific advantage disappears in the trainable LM setting

The trainable LM result is narrower than the proxy result.

The canonical interpretation supported by the LM increments is:
- fixed routing still works as a routing scaffold,
- but once expert weights are trained end-to-end, they adapt to whichever fixed scaffold they are given,
- so the trainable system no longer shows a meaningful quality distinction between Hopf routing and randomly permuted fixed routing.

This does not erase the proxy result. It changes the scope of the paper’s claim.

The clean interpretation for Paper 1 is:
- the proxy establishes that angular structure supports a concentrated fixed-routing footprint,
- the toy LM establishes that fixed routing can partially transfer without a learned gate matrix,
- the null result establishes that the trainable model does not preserve a geometry-specific quality advantage over a random fixed scaffold.

That is the honest boundary of the evidence.
