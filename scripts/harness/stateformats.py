"""State-format rule: delegate to the Rust implementation.

The format check itself lives in `crates/uor-r4-harness` (`contract.rs`), where D27 puts new tooling.
`scripts/harness/statecheck.py` was a second Python implementation of the same check and has been
removed — two implementations of one rule is how they drift apart, and it was new Python in a
repository that had just decided against it.

This module is the thin adapter the rule engine needs: it finds the built binary and runs
`uor-harness check`, reporting the result as a rule.

**Why it degrades instead of failing.** The binary is built by the `rust-harness` CI job, not by the
`harness` job where `check` runs. A missing binary means "not built here", which is not the same as
"the state is wrong" — so the rule reports that it was skipped rather than failing a check nobody
broke. The `rust-harness` job runs `check` after building, so the rule does execute in CI; it is only
in a bare Python-only environment that it stands down, and it says so.
"""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

BINARY_NAME = "uor-harness"


def find_binary(repo: Path) -> Path | None:
    """The built Rust binary, or None. Never builds — a rule must not compile anything."""
    candidates = []
    if env := os.environ.get("CARGO_TARGET_DIR"):
        candidates.append(Path(env) / "debug" / BINARY_NAME)
    candidates += [
        repo / "local" / "harness-target" / "debug" / BINARY_NAME,
        repo / "target" / "debug" / BINARY_NAME,
    ]
    for c in candidates:
        if c.is_file() and os.access(c, os.X_OK):
            return c
    return None


def check_contract(repo: Path, timeout: int = 120) -> tuple[str, str]:
    """Run the Rust format check. Returns (status, detail).

    status is one of `ok`, `drift`, `skipped`, `error`.
    """
    binary = find_binary(repo)
    if binary is None:
        return (
            "skipped",
            "the Rust harness is not built here, so the format check did not run "
            "(build it with `CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness`)",
        )
    try:
        proc = subprocess.run(
            # `check-formats`, NOT `check`. Python's `check` runs this rule; the Rust `check` is the
            # full 57-rule set and DELEGATES BACK to Python - so naming `check` here made the two
            # implementations call each other until the 120-second timeout fired. A recursion rather
            # than a wrong answer, which is the kinder of the two failures.
            [str(binary), "check-formats", "--repo", str(repo)],
            capture_output=True, text=True, timeout=timeout, check=False,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        return "error", f"could not run {binary.name}: {type(exc).__name__}: {exc}"
    out = (proc.stdout or "").strip()
    if proc.returncode == 0:
        first = out.splitlines()[0] if out else "every state file matches the frozen contract"
        return "ok", first
    if proc.returncode == 1:
        problems = [ln.strip() for ln in out.splitlines() if "PROBLEM" in ln]
        return "drift", "; ".join(problems[:3]) if problems else "state format drift"
    return "error", (proc.stderr or out or "the check exited non-zero").strip()[:300]


def register_rules():  # noqa: ANN201 - returns list[checkrules.Rule]
    from checkrules import Rule, RuleResult

    def _check(paths, reg=None):  # noqa: ANN001
        status, detail = check_contract(Path(paths.repo))
        if status == "ok":
            return [
                RuleResult(
                    rule_id="harness.state_formats",
                    severity="error",
                    ok=True,
                    message=detail,
                )
            ]
        if status == "skipped":
            return [
                RuleResult(
                    rule_id="harness.state_formats",
                    severity="info",
                    ok=True,
                    message=detail,
                    fix="run `cargo build -p uor-r4-harness` to enable this check",
                )
            ]
        return [
            RuleResult(
                rule_id="harness.state_formats",
                severity="error",
                ok=False,
                message=detail,
                path="docs/harness/state-formats.json",
                fix=(
                    "make the file match the contract, or re-freeze it deliberately and commit both "
                    "(see docs/harness/STATE-FORMATS.md)"
                ),
            )
        ]

    return [
        Rule(
            id="harness.state_formats",
            description="persisted state matches the frozen format contract",
            severity="error",
            check=_check,
            rationale=(
                "the state formats are the interface between the harness and the Rust port (D27); "
                "drift means the two implementations can disagree about what the shared record says. "
                "Checked by the Rust implementation, which is where D27 puts new tooling"
            ),
        )
    ]
