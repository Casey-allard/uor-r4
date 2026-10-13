# Final Rebuild Notes

## Major scientific and content upgrades

- Rebuilt the paper around a narrow empirical claim rather than a research-program narrative.
- Centered the manuscript on four evidentiary pillars only:
  - proxy-side sublinear routing from fixed angular geometry,
  - visible falsification controls on that proxy,
  - partial transfer to a toy trainable language model,
  - explicit null result against permuted fixed routing.
- Rewrote the method section so a technical reader can follow the mechanism from normalized input to routing address.
- Rewrote the setup section as a real scientific setup section rather than a memo summary.
- Made the null result part of the main scientific story instead of burying it in support material.

## Major visual and layout upgrades

- Replaced the previous conference-template-forward shell with a plainer arXiv-facing article shell calibrated against recent `cs.LG` papers.
- Kept the main-text float budget intentionally small and interpretable:
  - method schematic,
  - main scaling figure,
  - compact proxy-controls figure,
  - one combined LM table.
- Kept the appendix one-column and ordered by scientific purpose rather than by leftover assets.
- Used only generated paper-facing figures and tables backed by canonical analysis outputs under `paper1_assets/`.

## Shell and template choice

- Shell: plain `article` with restrained typography (`newtx`) and research-paper defaults.
- Reason: the benchmarked `cs.LG` papers were less template-branded and more authored in appearance than the earlier shell I used. For this manuscript, a calm one-column article form produced better method exposition, cleaner page composition, and a more credible arXiv-facing presentation than a conspicuous conference preprint shell.

## What was cut

- internal increment and project-history framing
- memo-like evidence-chain narration
- overclaiming language about replacing learned routing generally
- shell residue from previous failed paper attempts
- stale manuscript structure inherited from earlier drafts

## What was promoted to the appendix

- exact toy-LM setup table
- proxy control tables
- norm robustness table
- compact and full LM null/control tables
- auxiliary-loss comparator table
- PTB and WT2 detailed result tables

## Remaining weaknesses

- The strongest evidence remains the semantic proxy rather than the trainable LM.
- The trainable LM is still a toy setting and does not establish usefulness in modern large sparse transformers.
- The central negative result remains binding: Hopf is not better than permuted fixed routing after end-to-end training.
- A skeptical reviewer may still want a larger-scale trainable confirmation, but that is outside Paper 1’s evidence base.
