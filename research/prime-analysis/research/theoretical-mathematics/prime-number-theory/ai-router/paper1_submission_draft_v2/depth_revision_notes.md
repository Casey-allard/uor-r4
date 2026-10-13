# Paper 1 Depth Revision Notes

## What scientific content was added

- Expanded the method section from a compact algorithm description into a real mechanism explanation:
  - why the fixed 4D projection is used,
  - what the Hopf coordinates represent,
  - what the transport term changes,
  - why fixed sectorization can still produce concentration.
- Expanded the experimental section so it now reads like a scientific setup section rather than a short memo:
  - why the semantic proxy is the primary testbed,
  - exact toy-LM configuration,
  - routing budgets, datasets, and condition definitions,
  - the role of the effective routing footprint metric.
- Strengthened the semantic-proxy results with visible structural controls rather than brief references.
- Made the baseline-rationale argument explicit in the LM section instead of leaving it mostly implicit.

## What controls were promoted

Promoted into the main paper:
- structured vs column-permuted proxy control,
- Hopf vs adaptive `K`-means control,
- HOPF vs PERMUTED null result in the trainable LM,
- baseline rationale through the auxiliary-loss control discussion.

Promoted into the appendix as explicit tables:
- structured-vs-permuted proxy table,
- Hopf-vs-`K`-means proxy table,
- norm-invariance table,
- auxiliary-loss control table,
- full control/null-result table,
- PTB / WT2 detail tables,
- cross-dataset summary table,
- exact LM setup table.

## What visual support was added

- Kept the main scaling figure as the primary result figure.
- Kept the method schematic, but used it more meaningfully in the method exposition.
- Added a new proxy-control figure:
  - `figures/paper1_proxy_controls.pdf`
  - this makes the structure-destroying and comparator controls visible without forcing the reader to infer them from text alone.

## What still remains thin

- The LM evidence is still toy-scale and only covers FFN routing.
- The strongest scientific result remains the proxy scaling law.
- The paper still does not establish end-to-end hardware savings in a trained large-scale sparse model.
- The shell is improved, but it is still a stable custom two-column ML-style shell rather than an official venue template.
