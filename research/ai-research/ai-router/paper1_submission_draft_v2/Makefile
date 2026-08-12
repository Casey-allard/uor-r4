PYTHON ?= ../.venv-paper1/bin/python
TECTONIC ?= tectonic

.PHONY: all generate clean

all: generate
	$(TECTONIC) main.tex

generate:
	$(PYTHON) scripts/render_manuscript_inputs.py

clean:
	rm -f main.aux main.bbl main.blg main.log main.pdf
	rm -rf generated/*
