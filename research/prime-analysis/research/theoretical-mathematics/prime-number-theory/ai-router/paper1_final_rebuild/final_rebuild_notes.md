# Paper 1 Final Rebuild Notes

## Major scientific and content upgrades

- Rebuilt the manuscript around one narrow claim set only:
  - proxy-side sublinear routing from fixed angular geometry,
  - partial transfer to a toy trainable LM,
  - explicit null result against permuted fixed routing,
  - explicit limitation of scope.
- Replaced the prior memo-like exposition with a standard research-paper flow:
  - introduction,
  - background and evaluation scope,
  - method,
  - experimental setup,
  - proxy results,
  - LM results,
  - discussion and limitations,
  - conclusion,
  - appendix.
- Expanded the method section so the mechanism is intelligible rather than compressed:
  - intuition first,
  - schematic anchor figure,
  - coordinate definitions,
  - transport term interpretation,
  - sectorization logic,
  - paper-facing metric definition.
- Expanded setup and result interpretation so the paper explains why the proxy exists, what the toy LM can test, and how the null result changes the scope of the claim.

## Major visual and layout upgrades

- Switched to a NeurIPS-style preprint shell via `neurips_2023.sty`.
- Kept the main text visually disciplined:
  - Figure 1: method schematic,
  - Figure 2: main scaling figure,
  - Table 1: combined LM results.
- Moved all stronger controls and support tables into a deliberate one-column appendix.
- Kept figure and table inputs tied to generated canonical assets rather than hand-entered manuscript fragments.
- Normalized the method schematic to a PDF 1.5-compatible asset for cleaner manuscript production.

## Shell and build choice

- Shell: `article` + local `neurips_2023.sty` in preprint mode.
- Reason: it gives a recognizable ML-paper presentation, stable title/caption/section defaults, and an arXiv-friendly standalone package without depending on an external class install.
- Build path:
  1. `../.venv-paper1/bin/python scripts/render_manuscript_inputs.py`
  2. `tectonic main.tex`

## What was cut

- Increment-ID framing
- project-log / evidence-chain narration
- broad manifesto language
- over-compressed contribution bullets on page 1
- extra main-text floats that destabilized page composition
- dependence on hand-entered tables or hard-coded figure points

## What was promoted to the appendix

- detailed LM setup table
- structured-vs-permuted proxy control
- Hopf-vs-`K`-means proxy control
- norm robustness summary
- compact LM null-result table
- cross-dataset LM summary
- auxiliary-loss comparator control
- full control/null summary
- PTB and WT2 detail tables

## Remaining weaknesses

- The strongest evidence is still proxy-side.
- The trainable LM result remains toy-scale and FFN-only.
- The main negative result remains central: Hopf does not beat permuted fixed routing after end-to-end training.
- A few appendix tables remain visually dense, although they are ordered and readable.
