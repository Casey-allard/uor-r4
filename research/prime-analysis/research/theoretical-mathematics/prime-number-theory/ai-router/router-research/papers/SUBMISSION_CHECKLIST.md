# Submission Checklist — Angular Manifold Routing

**Paper:** Angular Manifold Routing: Sublinear Compute Reduction via Hopf-Base Sector Discretization  
**Target:** arXiv cs.LG (primary), cs.CL (secondary)  
**Bundle location:** `papers/`  
**Last updated:** 2026-03-26

---

## Status: READY TO SUBMIT (no blocking issues)

---

## 1. Manuscript Readiness

- [x] `papers/main.tex` — complete, all sections, all tables, figure reference
- [x] `papers/references.bib` — all citations present (TurboQuant, PTB, WT2, Switch, Hopf)
- [x] `papers/figures/fig_scaling_law.pdf` — generated, verified
- [x] `papers/Makefile` — `make all` + `make bundle` workflow documented
- [x] Abstract matches body: Tier 1 + Tier 2 claims only, no Paper 2+ content
- [x] Honest limitations Section 5.2: HOPF≈PERMUTED, toy-scale, native concentration regime
- [x] "Deeper computational substrate" sentence present in Section 5.3
- [x] INC-0172 KILL documented as architectural control (Section 4.4)
- [x] Cross-dataset table (Table 6): PTB + WT2, 2 seeds each
- [x] WT2 seed variance documented (Section 4.5 note)
- [x] Reproducibility: `hopf_routing_demo.py` referenced (numpy only, <60s)

## 2. Pre-Build Steps (one-time, on machine with pdflatex)

```bash
cd router-research/papers
make figures          # regenerates fig_scaling_law.pdf if needed
make all              # pdflatex + bibtex + 2× pdflatex
make check            # verify page count (target: 4-5 pages)
```

If no pdflatex: use Overleaf (upload main.tex + references.bib + figures/)
or install TeX Live: `brew install texlive` / `apt install texlive-full`

## 3. Zenodo Release (code + data snapshot)

**Do this before arXiv submission** — Zenodo timestamp precedes arXiv.  
**Bundle location:** `papers/release_bundle/`  
**Metadata draft:** `papers/ZENODO_METADATA.md`

### Step 1 — Prepare the zip (licenses are already in the bundle)

```bash
cd router-research/papers
zip -r angular_manifold_routing_bundle.zip release_bundle/
```

Bundle contains `LICENSE-CODE` (Apache-2.0) and `LICENSE-PAPER` (CC BY 4.0).  
No large data files — users generate `ppmi_proxy.npz` themselves via `generate_ppmi_proxy.py`.

### Step 2 — Upload to Zenodo

1. Go to https://zenodo.org/ → New Upload
2. Upload `angular_manifold_routing_bundle.zip`
3. Fill fields from `papers/ZENODO_METADATA.md`:
   - Title, author, description, keywords
   - License dropdown: **CC BY 4.0** (primary; Apache-2.0 for code noted in description)
4. **Reserve DOI before publishing** (Zenodo → Reserve DOI button)
5. ~~Add the reserved Zenodo DOI to `main.tex` and recompile PDF before publishing~~ — **DONE** (DOI `10.5281/zenodo.19243034` patched into `main.tex`; PDF rebuilt).
6. Replace `main.pdf` in the upload with the recompiled PDF (see `papers/main.pdf` and `papers/release_bundle/main.pdf`).
7. Publish the Zenodo record.

### Step 3 — Record the Zenodo DOI

- [ ] Add Zenodo DOI to `docs/research/ACTIVE_STATE.md`
- [ ] Add Zenodo DOI to `docs/DECISIONS.md`
- [ ] Update Section 4 below (arXiv comments field) with the Zenodo DOI

## 4. arXiv Submission Steps

**Do this after Zenodo is published** — include the Zenodo DOI in the comments field.

1. **Build the arXiv bundle:**
   ```bash
   make bundle
   # produces: angular_manifold_routing_arxiv.zip
   ```

2. **Upload to arXiv (https://arxiv.org/submit):**
   - Category: cs.LG (primary), cs.CL (secondary)
   - Files: `angular_manifold_routing_arxiv.zip` (contains .tex, .bbl, figures/)
   - Title: *Angular Manifold Routing: Sublinear Compute Reduction via Hopf-Base Sector Discretization*
   - Abstract: copy from `papers/main.tex` `\begin{abstract}...\end{abstract}`
   - Comments field: *Reproduction bundle: https://doi.org/10.5281/zenodo.19243034*
   - License: CC BY 4.0

3. **Verify PDF on arXiv preview** — check all tables render, figure displays, citations resolve.

4. **Record arXiv ID** in `docs/research/ACTIVE_STATE.md` and `DECISIONS.md`.

5. **Update Zenodo related identifiers** with the arXiv ID (Zenodo → Edit → Related identifiers).

## 5. Repo Snapshot Consistency

- [x] `make state` passes (validated 2026-03-26)
- [x] All increments INC-0171, INC-0172, INC-0173 closed with decisions in `DECISIONS.md`
- [x] `ACTIVE_STATE.md` points to publication phase, no active RR
- [x] `KILL_LIST_TRACKER.md` Stage 7: COMPLETE
- [x] `PUBLICATION_STRATEGY.md` reflects 2-seed WT2 confirm
- [ ] Verify `hopf_routing_demo.py` runs clean end-to-end on fresh checkout
- [ ] Add arXiv DOI/URL to `ACTIVE_STATE.md` after submission

## 6. Claim Scope Final Check

**In the paper (OK to claim):**
- eff_buckets = 2.957 × K^0.572 canonical law (169 increments, 7 seeds)
- Fixed Hopf routing replaces learned top-1 gating at 8% PPL cost (PTB + WT2, 2 seeds each)
- 1.39–1.56× efficiency vs dense routing (confirmed convergence values)
- HOPF ≈ PERMUTED in trainable LM (confirmed both datasets)
- Hardware proxy 3.0–4.9× (simulated, honestly disclosed)
- TurboQuant complementary framing (same geometric fact, opposite direction)

**NOT in the paper (withheld correctly):**
- Phase transport as independent LM contribution
- Spectral operator structure
- Sparse event routing mechanism
- H^4 × H^4 product architecture
- 10×–100× hardware savings

## 7. Post-Submission

- [ ] Update GitHub repo description/README to link arXiv paper
- [ ] Record submission date and arXiv ID in `ACTIVE_STATE.md`
- [ ] Add entry to `DECISIONS.md`: "Submitted to arXiv [date] [ID]"
- [ ] Open GitHub Issue for Paper 2 scoping (Phase transport / spectral paper) when ready

---

*This checklist supersedes the submission checklist in PUBLICATION_PACKET.md.*
