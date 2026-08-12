from __future__ import annotations

from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE_PATH = REPO_ROOT / "Makefile"
DETERMINISM_SCRIPT_PATH = REPO_ROOT / "scripts" / "determinism_local.sh"


def test_ci_local_pipeline_keeps_determinism_stage() -> None:
    makefile = MAKEFILE_PATH.read_text(encoding="utf-8")
    assert "ci-local: lint-local version-gate-local test-local determinism-local" in makefile


def test_determinism_local_script_runs_real_gate_suite_fail_fast() -> None:
    script = DETERMINISM_SCRIPT_PATH.read_text(encoding="utf-8")
    assert "set -euo pipefail" in script
    assert '[determinism-local] running real determinism gate suite' in script
    assert "python -m pytest --maxfail=1 -q" in script
    assert "tests/benchmark/test_real_determinism_gate.py" in script


def test_determinism_local_script_no_longer_uses_placeholder_scaffold() -> None:
    script = DETERMINISM_SCRIPT_PATH.read_text(encoding="utf-8")
    assert "test_determinism_scaffold.py" not in script
    assert "placeholder determinism test" not in script
