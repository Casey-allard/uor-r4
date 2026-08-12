## Master Spec Implementation Notes

Implemented from the frozen baseline bundle into a fresh two-column package.

### Main paper
- Two-column main paper shell with tight first page
- Frozen title used exactly
- Section order implemented exactly
- Page 1 limited to title, author/affiliation, abstract, and introduction
- Main-text float budget held to two figures and one table
- Problem-setting, method, setup, proxy results, LM results, discussion, conclusion, references preserved in the frozen order

### Main-text floats
- Figure 1: mechanism schematic
- Figure 2: main proxy scaling figure
- Table 1: main trainable language-model results

### Appendix
- `\clearpage`, `\onecolumn`, `\appendix` after references
- Appendix A: Detailed Experimental Setup
- Appendix B: Semantic Proxy Controls and Robustness
- Appendix C: Language-Model Support Tables

### Frozen text inserted
- Abstract replaced with the frozen abstract text, with only TeX-safe formatting
- Introduction replaced with the frozen introduction text
- Section 4 controls paragraph replaced with the frozen prose block
- Proxy-controls interpretation ending replaced with the frozen sentence
- Section 6.1 opening paragraph replaced with the frozen wording
- Section 6.4 comparator sentence replaced with the frozen wording
- Section 7 lead-in uses `The result remains limited in four respects:`
- Conclusion ending replaced with the frozen three-sentence paragraph

### Spec conflicts that were resolved conservatively
- The spec simultaneously required only two main-text figures and also supplied caption logic for both an occupancy figure and a separate scaling figure. To preserve the stricter float budget and keep the main result visible, the implementation keeps the scaling figure in the main text and moves the occupancy/control figure to Appendix B.
- The spec also simultaneously required one main-text table total and a caption rule that would make the routing procedure `Table 1`. To preserve the stricter one-table main-text budget, the combined LM results remain the only table in the main text and the routing procedure is moved to Appendix A.

