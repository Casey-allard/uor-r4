# Conversion Plan: From Internal Research Memo to arXiv-Friendly ML Paper

## Goal
Recast the existing manuscript into a conventional two-column ML paper that emphasizes a narrow empirical claim, neutral tone, standard section flow, and venue-native presentation.

## Main problems in the current draft
1. Internal project-history language (`INC-0171`, `INC-0173`, etc.) appears throughout the paper.
2. The paper is framed too strongly around the internal research program rather than the reader's problem.
3. Tone is occasionally argumentative or memo-like (`winning algorithm`, `survived all falsification attempts`, `correct baseline`).
4. The abstract and introduction try to carry too much total context.
5. Visual presentation is closer to a technical report than a conventional ML submission.

## Rewrite principles
- Use a two-column conference/journal-like layout.
- Keep the claim narrow and explicit.
- Present only final experimental evidence, not the whole project history.
- Treat TurboQuant as context, not as the spine of the paper.
- State the null result early and cleanly: the fixed-routing transfer survives, but geometry-specific quality gains do not in trainable LM settings.
- Remove project-log artifacts from the main text and appendix.

## Structural changes
### Title
Current title emphasizes “sublinear compute reduction” and Hopf-base sector discretization.

Rewrite keeps the core technical identity but simplifies the paper’s self-presentation:
- focus on fixed geometric routing from embedding angular structure
- avoid sounding like a full program manifesto

### Abstract
Rewrite to a conventional pattern:
- problem
- method
- main proxy result
- trainable LM result
- key limitation / null result

### Introduction
Rewrite around a standard motivating question:
- learned sparse routing is useful but adds learned gating machinery
- can some routing structure come from geometry alone?
- this paper studies that question in a proxy and a toy trainable LM

### Background
Shorten and depersonalize.
- mention TurboQuant once as context that normalized embeddings have exploitable angular structure
- do not foreground “independent parallel development”

### Method
Keep the actual algorithmic content.
- define the 4D projection
- define Hopf-base coordinates
- define transport correction
- define sector assignment

### Experiments
Split cleanly into:
- proxy experiments
- trainable LM experiments

### Results
Keep the central quantitative results, but remove internal labels and “evidence chain” feel.

### Discussion / Limitations
Retain honest limitations, especially:
- proxy strongest result, not large-scale LM proof
- toy trainable setting only
- Hopf not better than random fixed routing in trainable LM
- no attention-side routing
- no external replication

### Appendix
Drop the increment-history appendix entirely in the arXiv-friendly version.

## Tone conversion rules
Replace:
- “winning algorithm” -> “best-performing variant”
- “survived all falsification attempts” -> “performed best in our ablation study”
- “correct baseline” -> “appropriate comparison baseline for this setting”
- “independent parallel development” -> omit or minimize

## Visual conversion rules
- two-column layout
- tighter spacing and margins
- restrained title block
- no email address in the manuscript body
- standard table/figure captions
- denser but cleaner page texture

## What is preserved
- the core fixed-routing algorithm
- the proxy scaling-law result
- the PTB / WT2 transfer result
- the geometry-specific null result in trainable LM
- the narrow scope of the claim

## What is deliberately removed or suppressed
- increment IDs
- evidence-chain appendix
- project-history narration
- broader substrate language as a central framing device
- argumentative phrasing

## Deliverable produced
- a rewritten two-column LaTeX manuscript
- a compiled PDF showing the paper in a more venue-native format
