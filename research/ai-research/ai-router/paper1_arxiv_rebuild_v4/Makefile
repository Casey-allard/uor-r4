PYTHON := /Users/adminamn/ai-router/.venv-paper1/bin/python

.PHONY: all figures pdf clean

all: pdf

figures:
	$(PYTHON) scripts/render_manuscript_inputs.py

pdf: figures
	tectonic main.tex

clean:
	rm -f main.aux main.bbl main.blg main.log main.out main.pdf main.bcf main.run.xml
