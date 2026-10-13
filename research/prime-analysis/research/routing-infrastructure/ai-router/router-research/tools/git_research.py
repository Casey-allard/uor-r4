#!/usr/bin/env python3
"""Git-native research branch management tool.

This tool is the single entry point for all branch-context operations.
It is called automatically by the post-checkout git hook and can be run
directly from the Makefile or command line.

Commands
--------
    status              Print research context for the current branch.
    list                List all codex/ branches with kill-list stages.
    validate            Run check_research_state.py and return its exit code.
    new <RR> <SLUG>     Create a new codex/RR-###-slug branch.
        --inc <INC>     Also create an increment doc skeleton from the template.

Examples
--------
    python tools/git_research.py status
    python tools/git_research.py list
    python tools/git_research.py validate
    python tools/git_research.py new 061 shell-pressure-blend --inc 0137
    python tools/git_research.py new 069 product-phase-translation-eval
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

_ISSUE_REGISTRY = ROOT / "docs/program/ISSUE_REGISTRY.md"
_ACTIVE_STATE = ROOT / "docs/research/ACTIVE_STATE.md"
_INCREMENTS_DIR = ROOT / "docs/research/increments"


# ---------------------------------------------------------------------------
# Git helpers
# ---------------------------------------------------------------------------

def _git(cmd: list[str], check: bool = True, cwd: Path = ROOT) -> str:
    result = subprocess.run(
        ["git"] + cmd,
        capture_output=True,
        text=True,
        cwd=str(cwd),
    )
    if check and result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or result.stdout.strip())
    return result.stdout.strip()


def _current_branch() -> str:
    return _git(["rev-parse", "--abbrev-ref", "HEAD"], cwd=ROOT.parent)


def _all_codex_branches() -> list[str]:
    raw = _git(["branch", "--list", "codex/*"], cwd=ROOT.parent, check=False)
    branches = []
    for line in raw.splitlines():
        b = line.strip().lstrip("* ")
        if b:
            branches.append(b)
    return sorted(branches)


# ---------------------------------------------------------------------------
# Parsing helpers
# ---------------------------------------------------------------------------

def _extract_rr(text: str) -> str | None:
    """Extract the first RR-### pattern from a string."""
    m = re.search(r"RR-(\d+)", text)
    return f"RR-{int(m.group(1)):03d}" if m else None


def _read(path: Path) -> str:
    try:
        return path.read_text()
    except FileNotFoundError:
        return ""


def _extract_label(text: str, label: str) -> str | None:
    """Extract value from '- Label: `value`' or '- Label: value'."""
    pattern = re.compile(
        rf"^- {re.escape(label)}:\s*(?:`)?([^`\n]+)(?:`)?",
        re.MULTILINE,
    )
    m = pattern.search(text)
    return m.group(1).strip() if m else None


def _parse_active_state() -> dict[str, str]:
    text = _read(_ACTIVE_STATE)
    return {
        "rr": _extract_label(text, "Current primary RR") or "",
        "inc": _extract_label(text, "Current primary INC") or "",
        "stage": _extract_label(text, "Kill-list stage") or "",
    }


def _parse_registry_entry(rr_id: str) -> dict[str, str]:
    """Parse one RR entry from ISSUE_REGISTRY.md."""
    text = _read(_ISSUE_REGISTRY)
    # Match from the RR line to the next blank line or next '- `RR-' line
    pattern = re.compile(
        r"^- `" + re.escape(rr_id) + r"`.*?\n"
        r"((?:(?:  |\t).*\n?)*)",
        re.MULTILINE,
    )
    m = pattern.search(text)
    if not m:
        return {}

    full_block = m.group(0)
    body = m.group(1)
    data: dict[str, str] = {"rr": rr_id}

    # Title
    t = re.search(r"- Title: (.+)", full_block)
    data["title"] = t.group(1).strip() if t else ""

    # Labels  e.g. `[research][measure][active]`
    lm = re.search(r"`\[([^\]]+(?:\]\[?[^\]]*)*)\]`", full_block)
    if lm:
        data["labels"] = lm.group(0).strip("`")

    # Branch
    bm = re.search(r"- Branch: `(.+?)`", full_block)
    data["branch"] = bm.group(1).strip() if bm else ""

    # Canonical doc
    dm = re.search(r"- Canonical doc: `(.+?)`", full_block)
    data["doc"] = dm.group(1).strip() if dm else ""

    # Current continuation increment (may span two lines)
    cm = re.search(r"- Current continuation increment:\s*\n?\s*`(.+?)`", full_block)
    data["continuation"] = cm.group(1).strip() if cm else ""

    return data


def _parse_inc_doc(doc_rel: str) -> dict[str, str]:
    """Read key fields from an increment doc at a repo-relative path."""
    path = ROOT / doc_rel
    if not path.exists():
        return {"missing": "true"}

    text = path.read_text()

    def _first_line_after(heading: str) -> str:
        m = re.search(rf"^{re.escape(heading)}\s*\n(.*)", text, re.MULTILINE)
        return m.group(1).strip() if m else ""

    return {
        "status": _first_line_after("## Status"),
        "kill_stage": _first_line_after("## Kill-List Stage"),
        "success": _first_line_after("## Success Condition"),
    }


# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

def cmd_status(args: argparse.Namespace) -> int:  # noqa: ARG001
    """Print a concise research context panel for the current branch."""
    try:
        branch = _current_branch()
    except RuntimeError:
        print("(not inside a git repository)")
        return 1

    rr_id = _extract_rr(branch)
    active = _parse_active_state()

    w = 62
    sep = "=" * w

    print()
    print(sep)
    print(f"  BRANCH  : {branch}")

    if rr_id:
        entry = _parse_registry_entry(rr_id)
        if entry:
            title = entry.get("title", "")
            labels = entry.get("labels", "")
            print(f"  ISSUE   : {rr_id}  {title}")
            if labels:
                print(f"  LABELS  : {labels}")

            # Prefer continuation increment over canonical doc
            doc_rel = entry.get("continuation") or entry.get("doc")
            if doc_rel:
                inc_data = _parse_inc_doc(doc_rel)
                if inc_data.get("missing"):
                    print(f"  INC DOC : {doc_rel}")
                    print(f"  WARNING : Increment doc not found on disk")
                else:
                    print(f"  INC DOC : {doc_rel}")
                    stage = inc_data.get("kill_stage", "")
                    status = inc_data.get("status", "")
                    if stage:
                        # Only show the first line of the stage (skip "Primary:" prefix)
                        stage_line = next(
                            (l for l in stage.splitlines() if l.strip() and not l.strip().endswith(":")),
                            stage.splitlines()[0] if stage.splitlines() else stage,
                        )
                        print(f"  STAGE   : {stage_line.strip()}")
                    if status:
                        print(f"  STATUS  : {status}")
        else:
            print(f"  ISSUE   : {rr_id}  (not found in ISSUE_REGISTRY)")
    else:
        print(f"  ISSUE   : (no RR-### found in branch name)")

    # Active queue banner
    print()
    q_rr = active.get("rr", "?")
    q_inc = active.get("inc", "?")
    q_stage = active.get("stage", "?")
    print(f"  QUEUE   : {q_rr} / {q_inc}")
    print(f"  STAGE   : {q_stage}")

    if rr_id and active.get("rr") and rr_id != active["rr"]:
        print()
        print(f"  ! This branch ({rr_id}) is not the active queue ({active['rr']})")
        print(f"    To switch: git checkout codex/rr-{active['rr'].lower().replace('rr-', '')}-*")

    print(sep)
    print()
    print("  make state       validate all canonical docs")
    print("  make bootstrap   print full startup context")
    print("  make test        run test suite")
    print("  make branches    list all research branches")
    print()
    return 0


def cmd_list(args: argparse.Namespace) -> int:  # noqa: ARG001
    """List all codex/ branches with their RR, labels, and queue status."""
    try:
        branches = _all_codex_branches()
        current = _current_branch()
    except RuntimeError as e:
        print(f"git error: {e}")
        return 1

    active = _parse_active_state()
    active_rr = active.get("rr", "")

    print()
    print(f"  {'':2}{'BRANCH':<48}  {'RR':<8}  STATUS")
    print("  " + "-" * 70)

    for branch in branches:
        rr_id = _extract_rr(branch) or "—"
        marker = "► " if branch == current else "  "
        entry = _parse_registry_entry(rr_id) if rr_id != "—" else {}
        labels = entry.get("labels", "")
        # Derive a brief status token from labels
        for token in ("done", "active", "queued", "blocked"):
            if token in labels:
                status = token
                break
        else:
            status = ""
        queue_flag = " ◆ ACTIVE QUEUE" if rr_id == active_rr else ""
        print(f"  {marker}{branch:<48}  {rr_id:<8}  {status}{queue_flag}")

    print()
    print(f"  Active queue  : {active_rr} / {active.get('inc', '?')}")
    print(f"  Current stage : {active.get('stage', '?')}")
    print()
    return 0


def cmd_validate(args: argparse.Namespace) -> int:  # noqa: ARG001
    """Run check_research_state.py and return its exit code."""
    check = ROOT / "tools/check_research_state.py"
    if not check.exists():
        print(f"check_research_state.py not found at {check}")
        return 1
    result = subprocess.run([sys.executable, str(check)], cwd=str(ROOT))
    return result.returncode


def cmd_new(args: argparse.Namespace) -> int:
    """Create a new codex/RR-###-slug branch plus an optional increment doc."""
    # Normalise RR to three digits
    raw_rr = args.rr.upper().replace("RR-", "").lstrip("0") or "0"
    rr_id = f"RR-{int(raw_rr):03d}"
    slug = args.slug.lower().replace("_", "-")
    branch_name = f"codex/{rr_id.lower()}-{slug}"

    # Create the git branch
    try:
        _git(["checkout", "-b", branch_name], cwd=ROOT.parent)
        print(f"Created branch: {branch_name}")
    except RuntimeError as e:
        print(f"ERROR creating branch: {e}", file=sys.stderr)
        return 1

    # Optionally create a skeleton increment doc
    if args.inc:
        raw_inc = args.inc.replace("INC-", "").replace("INC_", "").lstrip("0") or "0"
        try:
            inc_int = int(raw_inc)
        except ValueError:
            print(f"Invalid INC number: {args.inc}", file=sys.stderr)
            return 1
        inc_id = f"INC-{inc_int:04d}"
        title_words = slug.replace("-", " ").title()
        fname = f"INC_{inc_int:04d}_{slug.replace('-', '_')}.md"
        doc_path = _INCREMENTS_DIR / fname

        if doc_path.exists():
            print(f"Increment doc already exists: {doc_path.relative_to(ROOT)}")
        else:
            active = _parse_active_state()
            stage = active.get("stage", "measure-consistent shell routing")
            doc_path.write_text(_inc_skeleton(rr_id, inc_id, title_words, stage))
            print(f"Created increment doc: {doc_path.relative_to(ROOT)}")

    print()
    print(f"Next:")
    print(f"  1. Edit the increment doc (if created) — fill in Hypothesis and Scope.")
    print(f"  2. Add {rr_id} to docs/program/ISSUE_REGISTRY.md.")
    print(f"  3. Run: make state  (to validate)")
    print()
    return 0


# ---------------------------------------------------------------------------
# Increment doc skeleton
# ---------------------------------------------------------------------------

def _inc_skeleton(rr_id: str, inc_id: str, title: str, stage: str) -> str:
    return f"""# {inc_id}: {title}

## Status
Queued next.

## Trigger
[Describe what makes this branch necessary — what previous result or
theory-level need triggers this increment.]

## Kill-List Stage
Primary:
1. {stage}

## Mathematical Object Under Test
[Specify the object. Examples:]
- routing manifold on the first `H^4`
- shell law / angular allocator
- Hopf-base vs fiber-phase split
- coupled field on second `H^4`
- phase transport law
- spectral / operator structure
- sparse event gate mechanism
- hardware cost surface

## Hypothesis
[State the mechanistic claim this branch tests. One or two sentences.]

## Minimal Scope
1. [Implementation step]
2. [Screen / confirm stage — use 1-seed screen, 2-seed confirm, 4-seed finalize]
3. Compare against:
   - [Reference 1]
   - [Reference 2]
4. Evaluate:
   - [Diagnostic 1]
   - [Diagnostic 2]

## Success Condition
[Concrete, measurable outcome that closes this branch positively.]

## Falsification Condition
[Concrete, measurable outcome that closes this branch negatively.]

## Acceptance
- one screen artifact exists in `results/analysis/`
- route-health diagnostics are interpretable
- an explicit keep/kill/refine decision is recorded

## Scope Guardrails
- [What this branch is NOT allowed to do]
- Do not open translated retrieval or sparse-event work unless the scope
  explicitly requires it.
- Do not claim hardware progress directly from this branch.

## Related
- Parent RR: `{rr_id}`
- Depends on: [previous INC or RR]
"""


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="git_research.py",
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    sub = parser.add_subparsers(dest="command")

    sub.add_parser("status", help="show research context for the current branch")
    sub.add_parser("list", help="list all codex/ branches")
    sub.add_parser("validate", help="run check_research_state.py")

    new_p = sub.add_parser("new", help="create a new research branch")
    new_p.add_argument("rr", help="RR number, e.g. 069 or RR-069")
    new_p.add_argument("slug", help="branch slug, e.g. product-phase-translation-eval")
    new_p.add_argument(
        "--inc",
        default=None,
        metavar="INC",
        help="INC number for an increment doc skeleton, e.g. 0138",
    )

    return parser


def main() -> int:
    parser = _build_parser()
    args = parser.parse_args()

    if args.command == "status" or args.command is None:
        return cmd_status(args)
    if args.command == "list":
        return cmd_list(args)
    if args.command == "validate":
        return cmd_validate(args)
    if args.command == "new":
        return cmd_new(args)

    parser.print_help()
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
