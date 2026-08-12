# Paper 1 Rebuild Spec

This spec is binding for the rebuild in `paper1_arxiv_rebuild/`.

## Final title direction

Use a specific, non-slogan title centered on the paper’s actual object:

**Fixed Geometric Routing from Angular Structure in Language Representations**

The title should foreground fixed routing and angular structure, not manifesto claims about replacing all MoE routing.

## Abstract target

The abstract must do five things in one compact block:

1. State the narrow question: can a fixed routing map derived from embedding geometry act as a routing scaffold?
2. Name the concrete mechanism: fixed 4D projection, Hopf-style coordinates, sectorization.
3. Report the main proxy result: sublinear effective routing footprint and persistence at large `K`.
4. Report the trainable LM transfer result and the null against permuted routing.
5. State the conclusion and limitation in the same paragraph.

## Exact section order

1. Introduction
2. Related Framing and Evaluation Scope
3. Fixed Geometric Routing Method
4. Experimental Setup
5. Semantic Proxy Results
6. Trainable Language-Model Results
7. Discussion and Limitations
8. Conclusion
9. References
10. Appendix

## Exact main-text float budget

Main text should contain only the assets needed to make the argument visible:

- Figure 1: method schematic
- Figure 2: main scaling figure
- Figure 3: compact proxy-controls figure
- Table 1: combined LM result table

No other main-text floats.

## Exact appendix contents

- Detailed LM setup table
- Proxy-controls figure repeated at readable appendix size if used in main text
- Structured-vs-permuted proxy table
- Hopf-vs-`K`-means proxy table
- Norm robustness table
- Compact LM null-result table
- Cross-dataset LM summary
- Auxiliary-loss comparator table
- Full null/control support table
- PTB detail table
- WT2 detail table

## Exact scientific scope

Keep the paper limited to:

- fixed geometric routing from embedding angular structure
- semantic proxy scaling result
- toy-LM transfer result
- null result against permuted fixed routing
- explicit limitation of scope

The paper must not claim:

- superiority over learned sparse routing in realistic LMs
- geometry-native hardware savings in deployed systems
- a broad theory of routing or representation geometry

## Exact controls that must be visible

Visible in the paper body or clearly surfaced through figure-plus-appendix support:

- structured vs permuted proxy control
- Hopf vs adaptive `K`-means proxy control
- HOPF vs PERMUTED null in the trainable LM
- no-aux baseline rationale via auxiliary-loss control

## Exact material to cut entirely

- increment IDs and research-program stage language
- evidence-chain / session-log narration
- manifesto framing
- broad hardware-efficiency promises unsupported by Paper 1 evidence
- stale `submit/` manuscript structure
- hand-entered result tables as scientific source of truth
- decorative or conference-template-forward visual decisions that make the paper look synthetic
