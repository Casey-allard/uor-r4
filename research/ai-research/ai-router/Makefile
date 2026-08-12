# Makefile — Router Research
# All important operations are available here so you never have to
# remember individual script paths or arguments.
#
# Usage:
#   make help          list all targets
#   make hooks         activate git hooks (one-time per clone)
#   make state         validate canonical research-state docs
#   make bootstrap     print full startup context packet
#   make branch        show research context for the current branch
#   make branches      list all codex/ branches
#   make test          run the full test suite
#   make baseline      run the kmeans baseline route
#   make pipeline      run full sweep → parse → summarize
#   make parse         parse raw logs only
#   make summarize     rebuild summary.csv only
#   make new           create a new research branch (see below)

SHELL   := /usr/bin/env bash
RD      := router-research

# ---------------------------------------------------------------------------
# Help
# ---------------------------------------------------------------------------
.PHONY: help
help:
	@echo ""
	@echo "Router Research — make targets"
	@echo "────────────────────────────────────────────────────────"
	@echo ""
	@echo "  Setup"
	@echo "    make hooks        activate .githooks/ for this clone (run once)"
	@echo ""
	@echo "  Research state"
	@echo "    make state        validate all canonical docs (ACTIVE_STATE,"
	@echo "                      KILL_LIST, ROUTE_MATRIX, PROJECT_BOARD, etc.)"
	@echo "    make bootstrap    print the full startup context packet"
	@echo "    make branch       show context for the current branch"
	@echo "    make branches     list all codex/ branches with kill-list stages"
	@echo ""
	@echo "  Development"
	@echo "    make test         run the full unittest suite"
	@echo "    make baseline     run kmeans baseline route"
	@echo "    make pipeline     full sweep → parse → summarize"
	@echo "    make parse        parse raw logs → results/parsed/"
	@echo "    make summarize    rebuild results/summary.csv"
	@echo ""
	@echo "  Branch management"
	@echo "    make new INC=0138 RR=069 SLUG=product-phase-translation-eval"
	@echo "             create branch codex/rr-069-product-phase-translation-eval"
	@echo "             and increment doc INC_0138_product_phase_translation_eval.md"
	@echo ""
	@echo "  Docs"
	@echo "    make onboarding   print GIT_ONBOARDING.md to terminal"
	@echo ""

# ---------------------------------------------------------------------------
# One-time setup
# ---------------------------------------------------------------------------
.PHONY: hooks
hooks:
	bash setup_hooks.sh

# ---------------------------------------------------------------------------
# Research state
# ---------------------------------------------------------------------------
.PHONY: state
state:
	cd $(RD) && python tools/check_research_state.py

.PHONY: bootstrap
bootstrap:
	cd $(RD) && python tools/context_bootstrap.py --group startup --cat

.PHONY: branch
branch:
	cd $(RD) && python tools/git_research.py status

.PHONY: branches
branches:
	cd $(RD) && python tools/git_research.py list

# ---------------------------------------------------------------------------
# Development
# ---------------------------------------------------------------------------
.PHONY: test
test:
	cd $(RD) && python -m unittest discover -s tests -v

.PHONY: baseline
baseline:
	cd $(RD) && bash runs/base_kmeans.sh

.PHONY: pipeline
pipeline:
	cd $(RD) && bash runs/run_pipeline.sh

.PHONY: parse
parse:
	cd $(RD) && python tools/parse_logs.py results/raw results/parsed

.PHONY: summarize
summarize:
	cd $(RD) && python tools/summarize.py results/parsed results/summary.csv

# ---------------------------------------------------------------------------
# Branch management
# ---------------------------------------------------------------------------
# Usage: make new INC=0138 RR=069 SLUG=product-phase-translation-eval
.PHONY: new
new:
ifndef RR
	$(error RR is required — e.g. make new RR=069 SLUG=product-phase-translation-eval)
endif
ifndef SLUG
	$(error SLUG is required — e.g. make new RR=069 SLUG=product-phase-translation-eval)
endif
ifdef INC
	cd $(RD) && python tools/git_research.py new $(RR) $(SLUG) --inc $(INC)
else
	cd $(RD) && python tools/git_research.py new $(RR) $(SLUG)
endif

# ---------------------------------------------------------------------------
# Docs
# ---------------------------------------------------------------------------
.PHONY: onboarding
onboarding:
	@cat $(RD)/docs/GIT_ONBOARDING.md
