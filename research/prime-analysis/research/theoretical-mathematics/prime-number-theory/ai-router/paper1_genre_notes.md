# Paper 1 Genre Notes

## Purpose of this note

This note records what recent serious `cs.LG` arXiv papers actually look like in practice, what the current Paper 1 lineage gets wrong, and what conventions the next rebuild must follow.

The benchmark pass for this note used:

- `arXiv:2603.26611` — *Benchmarking Tabular Foundation Models for Conditional Density Estimation in Regression*
- `arXiv:2603.26482` — *SPECTRA: An Efficient Spectral-Informed Neural Network for Sensor-Based Activity Recognition*
- `arXiv:2603.26339` — *Curvature-aware Expected Free Energy as an Acquisition Function for Bayesian Optimization*

These were examined at the PDF level, not just by title or metadata, because first-page composition, figure use, caption behavior, and section pacing matter.

## What a serious recent `cs.LG` arXiv paper looks like

### 1. The paper reads as authored scholarship, not as a template instance

The benchmarked papers do not look flashy or branded. They look plain, calm, and intentional. The shell does not draw attention to itself. The impression is that the authors wrote a paper, not that they filled a shell.

Practical consequence for Paper 1:

- avoid obvious conference-template theater if the actual prose and structure do not match it
- prefer a plain scholarly presentation whose authority comes from composition, density, and evidence

### 2. Titles are specific and technical, but not slogan-like

The title usually names the object, the method, or the task directly. It does not sound like a memo heading, a project codename, or a manifesto.

Good pattern:

- object + method + scope

Bad pattern for Paper 1:

- anything that sounds like internal program framing
- anything that promises architectural revolution the paper does not prove

### 3. The first page is dense, restrained, and credible

Across the sampled papers, page 1 has:

- a restrained title block
- normal author and affiliation formatting
- a dense abstract
- immediate movement into the introduction

The page is not sparse, theatrical, or overloaded with internal framing. It begins like a paper that expects to be read seriously.

### 4. Abstracts are compact and information-dense

The benchmarked abstracts do several things in one block:

- state the problem
- state the method or intervention
- state the experimental scope
- report the main result
- state the limitation or boundary when needed

They do not sound like internal pitch decks. They do not narrate “what this paper is trying to do” in casual voice.

### 5. Method exposition is legible before it becomes formal

The method sections in the benchmarked papers do not jump straight into compressed equations. They typically do:

1. say what the mechanism is for
2. explain the pieces in words
3. present the formal objects or equations
4. connect the equations back to interpretation

For Paper 1 this matters a lot, because a fixed Hopf-style routing map is not self-explanatory to a reader.

### 6. Experimental setup is boring in the good way

Serious papers include enough setup detail that a skeptical reader can tell:

- what data is used
- what the comparator is
- what metrics matter
- what the training or evaluation budget is
- what scope the setting is supposed to test

The setup section is not decorative. It earns trust.

### 7. Figures and tables are few, but each one does real work

The better benchmarked papers do not flood the main text with assets. But the assets they keep in the body are meaningful:

- a method figure that actually clarifies the mechanism
- a main result figure that is legible and information-rich
- tables whose captions tell the reader why the table matters

Captions are not repo-facing. They are paper-facing.

### 8. Controls are surfaced where they affect interpretation

Important controls do not disappear into “see appendix.” The main text usually presents either:

- the control itself, or
- a direct figure/table pointer with enough prose to explain how it changes interpretation

For Paper 1, this is essential because the claim only survives if the reader sees that:

- the proxy effect weakens under permutation
- Hopf beats adaptive `K`-means on the proxy
- Hopf loses its geometry-specific advantage in the trainable LM

### 9. Limitations are written in the same voice as the claims

The benchmarked papers do not switch into vague damage control when discussing limitations. They state them plainly, often close to the results.

That is especially important for Paper 1 because the null result is not incidental. It is part of the scientific story.

### 10. Appendices are structured support documents

The appendix usually has explicit purpose. It is not a loose pile of leftovers. It is where:

- exact settings live
- support tables live
- robustness checks live
- implementation details too bulky for main text live

The appendix should strengthen trust, not feel like an overflow buffer.

## What the repo’s original publication materials were implicitly targeting

The repo’s publication materials, especially:

- [PUBLICATION_STRATEGY.md](/Users/adminamn/ai-router/router-research/docs/research/PUBLICATION_STRATEGY.md)
- [PUBLICATION_PACKET.md](/Users/adminamn/ai-router/router-research/docs/research/PUBLICATION_PACKET.md)
- [ANGULAR_MANIFOLD_ROUTING_PAPER.md](/Users/adminamn/ai-router/router-research/docs/research/ANGULAR_MANIFOLD_ROUTING_PAPER.md)
- [SUBMISSION_CHECKLIST.md](/Users/adminamn/ai-router/router-research/papers/SUBMISSION_CHECKLIST.md)

were aiming at a short arXiv technical note whose main jobs were:

- establish timestamp priority after TurboQuant
- claim the narrow Tier 1 + Tier 2 result
- withhold the larger H^4 × H^4 program

That strategic narrowness is still correct. What is not correct is treating that as permission to produce a paper that looks under-authored or internal.

## What the current manuscript lineage gets wrong

The current rebuilt manuscript at [paper1_arxiv_rebuild/main.tex](/Users/adminamn/ai-router/paper1_arxiv_rebuild/main.tex) is better than the repo’s memo-like drafts, but it is still not acceptable as a serious academic submission. The problems are structural, stylistic, and visual.

### A. It still sounds like an internal project in too many places

Examples of unprofessional phrasing patterns still present:

- “Paper 1”
- “paper-facing metric”
- “This paper makes three claims, and only those three claims”
- “The most important limitation in the manuscript is also its most scientifically useful negative result”

These phrases reveal internal drafting scaffolding instead of sounding like authored scientific prose.

### B. The citation surface is too thin and too narrow

The current draft cites only a handful of works and does not read as if it sits in an active literature. The benchmarked papers use citations to locate the work in a field, not just to name the closest comparator and dataset sources.

For Paper 1, the current citation behavior makes the paper look under-read even if the underlying claim is real.

### C. The main-text figure and table strategy is still weak

The current draft’s figures are functional, but they do not yet look like figures from a serious research article:

- the schematic reads like a project diagram rather than a polished explanatory figure
- the proxy-controls figure is not yet publication-grade in composition or labeling
- the LM table is useful, but the overall visual rhythm still feels assembled rather than authored

### D. The prose still narrates the paper instead of simply being the paper

The current draft still explains what it is doing in meta terms. Strong papers usually avoid this unless the framing is itself scientifically necessary.

### E. The old repo publication target was too compressed

The repo repeatedly targets “4–5 pages” for a paper that actually needs:

- mechanism explanation
- proxy setup explanation
- comparator explanation
- null-result interpretation
- support appendix

That short-note target likely contributed to the memo-like compression that made the prior submission look unprofessional.

## Conventions the next rebuild must follow

### Manuscript identity

- It must read like a real research article, not like a compressed release note.
- It must stop using internal drafting language entirely.
- It must not say “Paper 1” inside the manuscript.

### Writing style

- Sentences should sound like a competent researcher writing for skeptical peers.
- Claims should be direct and scoped, not defensive and not overhyped.
- Limitations should be integrated naturally, not quarantined in apologetic language.

### Related-work and framing style

- The paper must be situated in learned sparse routing / MoE literature with enough citation density to feel literate.
- The TurboQuant framing should stay, but as context and urgency, not as a repeated slogan.

### Visual style

- figures must be explanatory, not merely present
- captions must explain why the figure matters
- all labels, axis text, and legends must be publication-scale and stylistically consistent

### Structure

- introduction
- framing / related context
- method
- experimental setup
- proxy results
- trainable LM results
- explicit null-result interpretation
- discussion and limitations
- conclusion
- structured appendix

### Evidence visibility

- the main text must show enough evidence that the claim does not depend on hidden trust
- the appendix must contain the exact support needed to defend the visible claims

## Practical takeaway

The next rebuild should not be a cosmetic revision of the current LaTeX. It should be a manuscript re-authorship pass informed by:

1. the repo’s actual publication strategy
2. the repo’s actual claim boundaries
3. the visual and rhetorical norms of real recent `cs.LG` papers

That means the next version must be less “paper-like” in the abstract and more recognizably like an authored research article in the concrete details of writing, citation, figure design, and section flow.
