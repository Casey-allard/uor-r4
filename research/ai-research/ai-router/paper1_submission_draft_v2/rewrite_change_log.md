# Paper 1 Rewrite Change Log

## Purpose

This directory contains the first professional submission-style rewrite of Paper 1. It is intentionally separate from the historical manuscript in `router-research/papers/`.

## What changed

- Rebuilt the manuscript as a self-contained two-column ML-style draft instead of a one-column research memo.
- Narrowed the paper to the scoped Paper 1 claims:
  - proxy-side sublinear scaling from fixed geometric routing,
  - partial transfer to a toy trainable LM,
  - null result against permuted fixed routing,
  - explicit limitations.
- Removed project-history framing:
  - no increment IDs,
  - no evidence-chain narration,
  - no manifesto language,
  - no internal “winning algorithm” or “survived falsification” rhetoric.
- Replaced dependence on hand-entered paper tables with generated LaTeX inputs derived from:
  - `paper1_assets/tables/*.csv`
  - `paper1_assets/data/scaling_fit_params.json`
- Replaced the old provisional scaling figure dependency with the generated vector figure from:
  - `paper1_assets/figures/paper1_scaling_main.pdf`
- Added a local build workflow:
  - `Makefile`
  - `scripts/render_manuscript_inputs.py`
  - local `references.bib`

## Why these changes were necessary

- The canonical manuscript in `router-research/papers/main.tex` contained strong evidence, but it still read like an internal memo:
  - one-column shell,
  - hand-entered result tables,
  - project-log framing,
  - overly broad interpretation sections.
- Paper 1 needs to function as a real submission artifact:
  - clean shell,
  - evidence-backed figures and tables,
  - explicit claim boundaries,
  - readable method explanation,
  - reproducible build path.

## Asset policy in this rewrite

- Evidence-bearing numeric content is sourced from generated canonical assets under `paper1_assets/`.
- The new manuscript does not use `submit/` or `preprints/` as evidence sources.
- The old paper under `router-research/papers/` was used only as an input reference for notation, citations, and the previously canonical method statement.

## Remaining limitations

- The new draft is only as strong as the current canonical evidence base:
  - strongest on the proxy result,
  - limited in the LM setting,
  - intentionally conservative in scope.
- The appendix currently includes only the generated tables already available in `paper1_assets/`; broader proxy-control appendix assets would require a separate generation pass if they are to be promoted into this draft.
