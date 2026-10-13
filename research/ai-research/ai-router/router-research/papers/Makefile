# Makefile for Angular Manifold Routing arXiv submission
# Usage:
#   make          — build main.pdf
#   make figures  — regenerate figures (requires matplotlib)
#   make bundle   — create arXiv submission zip
#   make clean    — remove build artefacts

MAIN    = main
FIGURES = figures/fig_scaling_law.pdf

.PHONY: all figures bundle clean check

# ── Default: build PDF ────────────────────────────────────────────────────────
all: $(MAIN).pdf

$(MAIN).pdf: $(MAIN).tex references.bib $(FIGURES)
	pdflatex -interaction=nonstopmode $(MAIN).tex
	bibtex $(MAIN)
	pdflatex -interaction=nonstopmode $(MAIN).tex
	pdflatex -interaction=nonstopmode $(MAIN).tex
	@echo "✓  $(MAIN).pdf built"

# ── Figures ───────────────────────────────────────────────────────────────────
figures: $(FIGURES)

$(FIGURES): generate_figures.py
	python generate_figures.py

# ── Consistency check (requires pdfinfo) ─────────────────────────────────────
check: $(MAIN).pdf
	@echo "Page count:"; pdfinfo $(MAIN).pdf | grep Pages
	@echo "File size:";  du -sh $(MAIN).pdf

# ── arXiv bundle ─────────────────────────────────────────────────────────────
# arXiv requires: .tex, .bbl (pre-compiled bibliography), figure files, no .pdf
bundle: $(MAIN).pdf
	@echo "Building arXiv bundle..."
	mkdir -p arxiv_bundle
	cp $(MAIN).tex arxiv_bundle/
	cp $(MAIN).bbl arxiv_bundle/
	cp -r figures arxiv_bundle/
	@# arXiv does not want the compiled PDF; remove if present
	rm -f arxiv_bundle/$(MAIN).pdf
	cd arxiv_bundle && zip -r ../angular_manifold_routing_arxiv.zip .
	@echo "✓  arxiv_bundle/ and angular_manifold_routing_arxiv.zip ready"
	@echo "   Upload angular_manifold_routing_arxiv.zip to arXiv"

# ── Clean ─────────────────────────────────────────────────────────────────────
clean:
	rm -f $(MAIN).aux $(MAIN).bbl $(MAIN).blg $(MAIN).log \
	      $(MAIN).out $(MAIN).pdf $(MAIN).toc
	rm -rf arxiv_bundle angular_manifold_routing_arxiv.zip
