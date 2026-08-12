SHELL := /bin/bash

.PHONY: lint-local version-gate-local test-local determinism-local ci-local

lint-local:
	@bash scripts/lint_local.sh

version-gate-local:
	@bash scripts/version_gate_local.sh

test-local:
	@bash scripts/test_local.sh

determinism-local:
	@bash scripts/determinism_local.sh

ci-local: lint-local version-gate-local test-local determinism-local
