"""Harness library: registry readers plus the task-facing modules.

Importable as ``scripts.harness.*`` from the repo root, or directly by adding
``scripts/harness`` to ``sys.path`` (what ``cli.py`` does).
"""

RELATIVE_MODULES = (
    "registries",
    "briefing",
    "mistakes",
    "prlint",
    "prep",
    "status",
    "receipts",
    "budget",
    "checkrules",
)

__all__ = ["core", *RELATIVE_MODULES]
