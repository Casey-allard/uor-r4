# Paper 1 v3 Rebuild Notes

## What changed scientifically

- Rewrote the paper around the actual supported claim rather than the repo's earlier release-note framing.
- Added real mechanism exposition:
  - why the router reads a fixed 4D chart,
  - how the paired-complex view is formed,
  - what the Hopf split does,
  - why the common phase is treated as a fiber variable,
  - what the transport term changes,
  - and how sectorization produces a route.
- Made the proxy/trainable distinction explicit instead of implicit:
  - the proxy tests the router as a geometric object,
  - the toy LM tests whether the fixed scaffold survives expert co-adaptation.
- Stated more clearly what the controls falsify:
  - generic fixed-partition explanations,
  - generic adaptive Euclidean clustering explanations,
  - a geometry-specific advantage after end-to-end training,
  - and the appropriateness of a load-balanced auxiliary-loss comparator.

## What changed visually

- Dropped the `newtx` look and moved back to a calmer article shell with Latin Modern.
- Replaced block-like diagrams with new data-driven figures:
  - a mechanism figure that shows chart selection, paired complex coordinates, Hopf base / fiber split, and sectorization,
  - a proxy geometry figure built directly from the released `ppmi_proxy.npz` artifact,
  - a stronger scaling figure with a second panel showing how the occupancy gap grows,
  - an LM-dynamics figure showing why the trainable null matters.
- Kept the appendix as structured support rather than a dump bucket.

## What was cut or demoted

- Internal manuscript language such as "Paper 1" and "paper-facing".
- Repo-history narration and publication-packet tone.
- Broad architecture claims not supported by this paper.
- Decorative control plots that did not teach the mechanism.

## What remains imperfect

- The method figure is much better than the earlier block diagrams, but it could still be refined further by a human illustrator.
- The static proxy remains the strongest evidence source.
- The trainable model remains toy-scale and FFN-only.
- The bibliography/build path still relies on BibTeX under `tectonic`, which compiles successfully but produces rerun warnings.
