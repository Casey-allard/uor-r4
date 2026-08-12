# Final Rebuild Notes

## What changed scientifically

- Rebuilt the manuscript around the repo's actual public claim boundary rather than the broader internal research program.
- Treated the semantic proxy as the primary evidence layer and the trainable language model as a secondary transfer layer.
- Promoted the null result against permuted fixed routing into the core scientific argument instead of leaving it as damage control.
- Kept the scope explicitly narrow:
  - fixed routing from angular structure,
  - proxy-side sublinear routing,
  - toy-LM transfer,
  - no claim of geometry-specific superiority after end-to-end training.

## What changed in presentation

- Replaced the earlier memo-like wording with normal research prose.
- Removed internal phrases such as "Paper 1," "paper-facing," increment language, and project-log narration.
- Expanded the citation surface so the paper reads as situated in the sparse-routing and representation-geometry literature rather than as a repo artifact with a few supporting citations.
- Rebuilt the method schematic as a cleaner explanatory figure and regenerated the proxy-controls figure with a more publication-oriented design.
- Regenerated the main scaling figure locally from canonical exported data in a cleaner plot style.

## Shell choice

- Shell: plain one-column `article` with restrained typography.
- Reason: after benchmarking recent `cs.LG` papers, the better target for this manuscript was not a louder conference shell but a calmer authored article form that gives the method and setup enough room to read credibly.

## What was cut

- internal program framing
- explicit "independent parallel development" narration
- repo-facing provenance language inside captions and body text
- broader architectural claims not supported by the evidence in this paper

## What remains in the appendix

- exact toy-LM setup table
- proxy control figure and support tables
- normalization robustness table
- compact and full null/control support tables
- auxiliary-loss comparator table
- PTB and WT2 detail tables

## Remaining weaknesses

- The proxy result is still stronger than the trainable result.
- The trainable LM remains toy-scale.
- The central negative result remains binding: fixed Hopf routing is not better than permuted fixed routing after end-to-end learning.
- The paper is now much more credible as a manuscript object, but it still depends on a skeptical reader accepting the proxy as the right place to test the geometric hypothesis first.
