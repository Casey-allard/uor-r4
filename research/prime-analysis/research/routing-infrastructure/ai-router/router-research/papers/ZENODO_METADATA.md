# Zenodo Deposit Metadata — Angular Manifold Routing

> **Prepare this file before uploading to Zenodo.**  
> Fill in the `[FILL IN]` fields and finalize the license section before uploading.

---

## Title

Angular Manifold Routing: Sublinear Compute Reduction via Hopf-Base Sector Discretization

---

## Authors

| Field | Value |
|---|---|
| Name | Allard, Casey |
| Affiliation | Independent Researcher |
| Email | caseyallard@hotmail.com |
| ORCID | https://orcid.org/0009-0005-9758-5969 |

---

## Description / Abstract

*(Copy the paper abstract here before uploading. The version below is the locked submission abstract.)*

We show that the same angular non-uniformity of L2-normalized token embeddings that enables TurboQuant's extreme data compression also enables sublinear routing computation in transformer-style architectures. A fixed Hopf fibration map exploits this structure to produce a routing footprint scaling as K^0.572 vs K^1.0 for dense routing — an advantage that persists at K=5000 (ratio 2.6–2.8×). In a 2-layer trainable language model, fixed geometric routing replaces a learned top-1 gate with only 8% validation perplexity cost and no learned gate matrix, while using 46 of 64 effective expert paths at convergence (1.4× more efficient than dense routing). A second-dataset replication on WikiText-2 (confirmed 2 seeds) finds a HOPF/BASELINE ratio of 1.081 — numerically identical to the PTB confirmed ratio — under identical training conditions. This result is scoped to the 2-layer toy-scale trainable setting and should not be read as a claim of broad MoE replacement or large-scale transformer substitution. Taken together with TurboQuant, this work suggests the angular non-uniformity of embeddings has engineering consequences in both data compression and routing computation.

---

## Keywords

*(Enter these as individual keywords in Zenodo's keyword field.)*

- transformer routing
- Hopf fibration
- mixture of experts
- sparse routing
- angular non-uniformity
- sublinear compute
- language model efficiency
- geometric deep learning
- TurboQuant

---

## Upload Type

Software + Publication (type: preprint)

---

## License

This bundle uses a split license structure:

| Component | License |
|---|---|
| Manuscript (`main.pdf`, `main.tex`, `references.bib`, `figures/`) | **CC BY 4.0** — Creative Commons Attribution 4.0 International |
| Code / scripts (`hopf_routing_demo.py`, `generate_ppmi_proxy.py`, `generate_figures.py`) | **Apache-2.0** |

**Zenodo dropdown:** Select **CC BY 4.0** (the primary artifact is the manuscript).  
In the Zenodo Notes/Description field, add: *"Code files in this bundle are licensed under Apache-2.0; see `LICENSE-CODE`."*  
Full license texts are in `LICENSE-CODE` and `LICENSE-PAPER` within the bundle.

---

## Related Identifiers

*(Zenodo does not require an arXiv ID before publishing. Add the arXiv ID as a Zenodo record update after arXiv submission.)*

| Relationship | Identifier | Note |
|---|---|---|
| Is supplemented by | `[add arXiv ID after submission]` | arXiv preprint |
| Is source code of | `[add arXiv ID after submission]` | Reproduction bundle for the paper |

---

## Access Rights

Open access (recommended for public preprint release)

---

## Journal / Conference

Leave blank (this is a preprint deposit)

---

## Version

1.0.0 — Initial release (prior to arXiv submission)

---

## Notes / Additional Description

*(Optional field in Zenodo. Suggested text:)*

> This record contains the reproduction bundle for the preprint "Angular Manifold Routing: Sublinear Compute Reduction via Hopf-Base Sector Discretization." The bundle includes the manuscript PDF, LaTeX source, bibliography, figure generation code, and a numpy-only standalone demo that reproduces Table 3 of the paper. The demo requires a PPMI-SVD embedding built from the public WikiText-2 corpus; a generation script (generate_ppmi_proxy.py) is included. The full experimental increment records are available in the preprint's artifact bundle.

---

## Checklist Before Uploading

- [x] Licenses locked: `LICENSE-CODE` (Apache-2.0) and `LICENSE-PAPER` (CC BY 4.0) in bundle
- [ ] `CITATION.cff` or BibTeX citation block finalized (see README.md in bundle)
- [ ] Zip the bundle: `cd papers && zip -r angular_manifold_routing_bundle.zip release_bundle/`
- [ ] Upload zip to https://zenodo.org/ under "New upload"
- [ ] Reserve DOI before publishing (Zenodo allows pre-reservation)
- [x] Add Zenodo DOI to paper and recompile: `https://doi.org/10.5281/zenodo.19243034`
- [ ] Publish Zenodo record
- [ ] Record Zenodo DOI in `docs/research/ACTIVE_STATE.md` and `docs/DECISIONS.md`
- [ ] After arXiv submission: update Zenodo related identifiers with arXiv ID
