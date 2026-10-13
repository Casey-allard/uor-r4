"""Pre-registration before compute, and the frozen acceptance panels.

D22 made three things mechanical, and this module is those three things as code:

1. **Pre-register before compute.** A run gets a sheet (`docs/harness/prep/*.md`) that
   states the bar, the base artifact, the data split, the steps, the stop rule and the
   full cost *before* it starts ([session-goal.md](../../docs/labs/session-goal.md) clause 4).
2. **Minimum test power.** A decisive run uses >= 2 seeds or >= 200 fresh rows, the
   mechanism is trained in rather than bolted on, and no bar is narrower than the panel's
   stated noise (D22 section 3). The sheet carries that block explicitly.
3. **TEST FITNESS before model compute.** The sheet ends with the literal line
   ``TEST FITNESS: PENDING``; the FIT review replaces it with ``FIT`` or ``NOT_FIT``.
   A sheet with a run plan and a PENDING/NOT_FIT verdict is not cleared for compute
   (rule ``prep.fit_before_run``).

Nothing here posts to GitHub by default. ``fit_review`` prints the exact comment text;
``--post`` is opt-in and uses ``gh issue comment``.

Panels live in ``docs/harness/panels/*.json`` and are frozen: the bar a run is scored
against must exist before the run. A measured value inside the panel's stated noise is
reported ``AT NOISE`` and is never a real difference.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

from core import HarnessPaths, emit, today
from checkrules import Rule, RuleResult, register as register_rule
from registries import Panel, Registry, find_panel
from util import ABSENT, MISSING, read_text, truncate

# --------------------------------------------------------------------------- constants

TEST_FITNESS_PENDING = "TEST FITNESS: PENDING"
TEST_FITNESS_FIT = "TEST FITNESS: FIT"
TEST_FITNESS_NOT_FIT = "TEST FITNESS: NOT_FIT"
VERDICTS = ("FIT", "NOT_FIT")

#: D22 section 3, the minimum test power every decisive run must state.
MIN_SEEDS = 2
MIN_ROWS = 200

#: owner rules that apply to every pre-registration (docs/harness/milestones.json).
APPLICABLE_RULES = ("D21", "D22", "D23", "D24")

#: the cost a run must charge, in the session-goal contract's own breakdown.
COST_BUCKETS = ("preparation", "run", "controls", "evaluation", "retries", "storage")

#: owner caps (AGENTS.md "GPU pods", docs/labs/compute.md section "Caps").
POD_CAP = 4
RATE_CAP_USD_PER_H = 8.00

#: free disk the owner requires locally (AGENTS.md resources section), and the
#: storage stop margin kept for retained evidence (standing authorization 2026-09-06).
DISK_BAND_GB = (30, 70)
STORAGE_MARGIN_MIB = 128

_H_RE = re.compile(r"^(#{2,4})\s+(.+?)\s*$", re.MULTILINE)
# A verdict line starts a line ("TEST FITNESS: PENDING" / "TEST FITNESS: FIT (reviewer x)").
# Anchoring matters: "carry a TEST FITNESS: FIT review on the milestone" is prose in the
# owner-rules block, not a verdict, and must never be read as one.
_TF_RE = re.compile(r"^[ \t]*TEST FITNESS:[ \t]*(PENDING|FIT|NOT_FIT)(?![A-Za-z_])", re.IGNORECASE | re.MULTILINE)


# ------------------------------------------------------------------------------ sheets


def prep_dir(paths: HarnessPaths) -> Path:
    return paths.harness / "prep"


def sheet_files(paths: HarnessPaths) -> list[Path]:
    """Every pre-registration sheet, newest name first (the name carries the date)."""
    d = prep_dir(paths)
    if not d.is_dir():
        return []
    return sorted(d.glob("*.md"), reverse=True)


def slug(text: str) -> str:
    s = re.sub(r"[^a-z0-9]+", "-", str(text).strip().lower()).strip("-")
    return s or "unnamed"


def sections(text: str) -> list[tuple[str, str]]:
    """Split a sheet into (heading, body) pairs at ## / ### / #### level."""
    marks = list(_H_RE.finditer(text))
    out: list[tuple[str, str]] = []
    for i, m in enumerate(marks):
        end = marks[i + 1].start() if i + 1 < len(marks) else len(text)
        out.append((m.group(2).strip(), text[m.end() : end].strip()))
    return out


def test_fitness(text: str) -> str:
    """PENDING | FIT | NOT_FIT, or ABSENT when the sheet has no verdict line."""
    m = _TF_RE.search(text)
    return m.group(1).upper() if m else ABSENT


def _field(text: str, label: str) -> str:
    m = re.search(rf"^\s*[-*]?\s*{re.escape(label)}\s*:\s*(.+?)\s*$", text, re.MULTILINE | re.IGNORECASE)
    return m.group(1).strip() if m else ""


def _has_section(found: list[tuple[str, str]], *needles: str) -> bool:
    for heading, _body in found:
        low = heading.lower()
        if all(n in low for n in needles):
            return True
    return False


def _has_run_plan(text: str) -> bool:
    """A sheet plans a run when it names one, or carries a steps/cost block."""
    if _field(text, "Run"):
        return True
    found = sections(text)
    return any(
        ("step" in h.lower()) or ("cost" in h.lower()) or ("run" in h.lower()) for h, _ in found
    )


def sheet_requirements(paths: HarnessPaths) -> dict[str, Any]:
    """The required blocks, so the rule and the renderer cannot drift apart.

    Each entry is (key, human name, predicate over the sheet's sections + body).
    """
    return {
        "bar": ("Bar", lambda secs, body: _has_section(secs, "bar")),
        "base": ("Base artifact", lambda secs, body: _has_section(secs, "base")),
        "split": ("Data split", lambda secs, body: _has_section(secs, "split")),
        "steps": ("Steps", lambda secs, body: _has_section(secs, "step")),
        "stop": ("Stop rule", lambda secs, body: _has_section(secs, "stop")),
        "cost": ("Cost", lambda secs, body: _has_section(secs, "cost")),
        "power": ("Minimum test power", lambda secs, body: _has_section(secs, "power")),
        "fit": ("TEST FITNESS line", lambda secs, body: test_fitness(body) != ABSENT),
    }


# ------------------------------------------------------------------------- registries


def _mechanism_path(paths: HarnessPaths, mechanism: str) -> str:
    if not mechanism:
        return MISSING
    candidate = paths.mechanisms_dir / f"{mechanism}.md"
    rel = f"docs/mechanisms/{mechanism}.md"
    if candidate.is_file():
        return rel
    known = sorted(p.stem for p in paths.mechanisms_dir.glob("*.md")) if paths.mechanisms_dir.is_dir() else []
    return f"{MISSING} (no brief at {rel}; known: {', '.join(known) or 'none'})"


def _panel_lines(panel: Panel) -> list[str]:
    out: list[str] = []
    for bar in panel.bars:
        bid = str(bar.get("id", "?"))
        label = str(bar.get("label", bid))
        current = bar.get("current")
        target = bar.get("target")
        of = bar.get("of")
        suffix = f"/{of}" if of not in (None, "") else ""
        unit = bar.get("unit", "")
        direction = bar.get("direction")
        tail = f" [{direction}]" if direction else ""
        out.append(f"  - {bid}: {label} {current}{suffix} -> {target}{suffix} {unit}{tail}".rstrip())
    if not out:
        out.append(f"  - {MISSING}: the panel has no bars")
    return out


def _rule_index(reg: Registry) -> dict[str, str]:
    return {str(r.get("id", "")): str(r.get("rule", "")) for r in reg.milestones.owner_rules}


# ------------------------------------------------------------------------------ render


def _bar_block(args: Any, milestone: Any, panel: Panel | None) -> list[str]:
    lines = ["## Bar (pre-registered)"]
    if getattr(args, "bar", None):
        lines.append(f"- bar: {args.bar}")
    else:
        lines.append(f"- bar: {MISSING} - a run without a pre-registered bar cannot be scored")
    if panel is not None:
        lines.append(f"- panel: `{panel.id}` (grader: {panel.grader or MISSING}, frozen: {'yes' if panel.frozen else 'NO'})")
        lines.append(f"- panel noise: {panel.noise or MISSING}")
        lines.extend(_panel_lines(panel))
        lines.append("- no bar in this sheet may be narrower than the panel noise above (D22 section 3).")
    else:
        lines.append(f"- panel: {MISSING} - the milestone names no frozen panel")
    return lines


def _power_block(args: Any) -> list[str]:
    """D22 section 3 as an explicit, checkable block."""
    seeds = getattr(args, "seeds", None)
    rows = getattr(args, "rows", None)
    decisive = bool((seeds or 0) >= MIN_SEEDS or (rows or 0) >= MIN_ROWS)
    lines = [
        "## Minimum test power (D22 section 3)",
        f"- seeds: {seeds if seeds is not None else MISSING} (>= {MIN_SEEDS} seeds for a decisive run)",
        f"- fresh rows: {rows if rows is not None else MISSING} (>= {MIN_ROWS} fresh rows for a decisive run)",
        f"- decisive run: {'YES' if decisive else 'NO'} - D22 section 3 needs >= 2 seeds OR >= 200 fresh rows"
        + ("" if decisive else " (raise --seeds or --rows before compute)"),
        f"- trained in rather than bolted on: {'YES' if decisive else MISSING} - state the pretraining or the full fine-tune "
        "that gives the base time to adapt (a short add-on is a probe, not a decisive run)",
        "- fresh-transfer plan for a KEEP: MISSING - every development KEEP also reports a fresh, held-out transfer number (D22 section 4)",
        "- noise floor: the bars above must not be narrower than the panel's stated noise; a single-row bar is not a bar",
    ]
    return lines


def _cost_block(args: Any, mechanism: str, milestone: Any) -> list[str]:
    supplied = str(getattr(args, "cost", None) or "").strip()
    token = supplied or MISSING
    model_work = _looks_like_model_work(args, mechanism)
    lines = [
        "## Cost (charged cumulatively, before execution)",
        f"- preparation: {token} (context/data windows, threads, peak RAM)",
        f"- run: {token} (wall time and steps; the whole job, not the first attempt)",
        f"- controls: {token} (the no-mechanism control at the same recipe, steps and seeds)",
        f"- evaluation: {token} (each checkpoint scored, plus the fresh held-out draw for a KEEP)",
        f"- retries: {token} (diagnose, correct and retry within the remaining budget; a self-set budget is not a stop rule)",
        f"- storage: {token} (temporary, retained, and the {STORAGE_MARGIN_MIB} MiB storage stop margin)",
    ]
    if model_work:
        lines.append(
            "- placement (D24): model compute is GPU work only - lease the GPUs you use through "
            "`scripts/pod/uor-pod run POD --lab L --session S --gpu K -- CMD`; no CPU model compute "
            "and no host fallback (a fallback is a defect to fix, not a cost to report)"
        )
    else:
        lines.append(
            "- placement (D24): no model compute is planned; CPU-bound preparation, grading and "
            "checks run on the owner's laptop, and no GPU is requested for work that cannot change a decision"
        )
    lines.append(
        "- pod/lease plan: MISSING - name the pod, the GPUs leased, `--hours` set to the whole job plus "
        "margin, the renew cadence, and the release-then-down step"
    )
    lines.append(
        f"- owner caps: <= {POD_CAP} running pods and <= ${RATE_CAP_USD_PER_H:.2f}/h across all labs, all sessions; "
        "no new spending class without an owner decision (AGENTS.md, docs/labs/compute.md section Caps)"
    )
    return lines


def _looks_like_model_work(args: Any, mechanism: str) -> bool:
    text = " ".join(
        str(getattr(args, k, "") or "") for k in ("run", "steps", "bar", "mechanism", "stop")
    ).lower()
    return any(t in text for t in ("train", "fit", "finetune", "fine-tune", "gpu", "cuda", "seed", "checkpoint"))


def render_sheet(args: Any, paths: HarnessPaths, reg: Registry, milestone: Any, panel: Panel | None) -> str:
    """The sheet text. Every field is either supplied, read from the registry, or MISSING."""
    mechanism = str(getattr(args, "mechanism", None) or "")
    run = str(getattr(args, "run", None) or MISSING)
    line = milestone.lines[0] if milestone.lines else {}
    line_name = str(line.get("name", MISSING))
    line_count = line.get("count", MISSING)
    line_of = line.get("of", 3)
    rules = _rule_index(reg)

    out: list[str] = [
        f"# Pre-registration - {milestone.id} ({mechanism or MISSING})",
        "",
        f"- date: {today()}",
        f"- milestone: {milestone.id} #{milestone.issue} - {milestone.title}",
        f"- mechanism: {mechanism or MISSING}",
        f"- mechanism brief: {_mechanism_path(paths, mechanism)}",
        f"- run: {run}",
        f"- line: {line_name} - count {line_count}/{line_of} (D21/D22 count is owned by the milestone, not this sheet)",
        "",
    ]
    out += _bar_block(args, milestone, panel)
    out += [
        "",
        "## Base artifact",
        f"- base: {getattr(args, 'base', None) or milestone.base_artifact or MISSING}",
        "",
        "## Data split",
        f"- split: {getattr(args, 'split', None) or MISSING}",
        "- training data never includes the frozen panel rows; scoring uses the frozen panel and, for a KEEP, a fresh held-out draw (D22 section 4)",
        "",
        "## Steps",
        f"- steps: {getattr(args, 'steps', None) or MISSING}",
        "",
    ]
    out += _power_block(args)
    out += [
        "",
    ]
    out += _cost_block(args, mechanism, milestone)
    out += [
        "",
        "## Stop rule",
        f"- stop: {getattr(args, 'stop', None) or MISSING}",
        "- self-set step/wall budgets are estimates, not stop rules: extend and renew rather than cancel a healthy run; the only hard stops are the owner caps, the disk floor and a failed or diverged run",
        "",
        "## Owner rules that apply",
    ]
    for rid in APPLICABLE_RULES:
        text = rules.get(rid)
        out.append(f"- {rid}: {text}" if text else f"- {rid}: {MISSING} from docs/harness/milestones.json")
    out += [
        "- D22: a negative closes this configuration only, never the mechanism; stopping a mechanism needs the owner.",
        "",
        "## Panel target (milestone registry)",
        f"- headline: {milestone.headline_line()}",
        f"- panel: {milestone.panel or MISSING}",
        f"- source: docs/harness/milestones.json (issue #{milestone.issue})",
        "",
        "## Verdict",
        TEST_FITNESS_PENDING,
    ]
    return "\n".join(out) + "\n"


# ------------------------------------------------------------------------------ actions


def _default_sheet_path(paths: HarnessPaths, milestone: Any, mechanism: str) -> Path:
    return prep_dir(paths) / f"{slug(milestone.key)}-{slug(mechanism or 'no-mechanism')}-{today()}.md"


def _record(paths: HarnessPaths, **fields: Any) -> dict[str, Any] | None:
    """A prep receipt. Never fatal: the sheet matters more than its receipt."""
    try:
        from receipts import record  # type: ignore import-not-found
    except ImportError:
        return None
    try:
        return record(paths, "prep", **fields)
    except (ValueError, OSError):  # a broken ledger must not lose the sheet
        return None


def new_sheet(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    key = str(getattr(args, "milestone", "") or "")
    if reg is None or reg.milestones is None:
        emit("the milestone registry did not load; a sheet cannot be assembled", sys.stderr)
        return 1
    milestone = reg.milestones.get(key)
    if milestone is None:
        known = ", ".join(sorted(reg.milestones.by_key))
        emit(f"unknown milestone {key!r}; known: {known}", sys.stderr)
        return 2
    mechanism = str(getattr(args, "mechanism", None) or "")
    panel = _panel_for_milestone(reg, milestone.panel)

    text = render_sheet(args, paths, reg, milestone, panel)
    write = getattr(args, "write", None)
    target = Path(write) if write else _default_sheet_path(paths, milestone, mechanism)
    if not target.is_absolute():
        target = paths.repo / target
    try:
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")
    except OSError as exc:
        emit(f"cannot write {target}: {exc}", sys.stderr)
        return 1

    rel = _rel(target, paths.repo)
    rec = _record(
        paths,
        milestone=milestone.key,
        mechanism=mechanism,
        run=str(getattr(args, "run", None) or ""),
        bar=str(getattr(args, "bar", None) or ""),
        seeds=getattr(args, "seeds", None),
        rows=getattr(args, "rows", None),
        sheet=rel,
        verdict="PENDING",
    )
    if getattr(args, "json", False):
        emit(
            json.dumps(
                {
                    "sheet": rel,
                    "milestone": milestone.key,
                    "mechanism": mechanism,
                    "panel": panel.id if panel else "",
                    "verdict": "PENDING",
                    "receipt": rec,
                    "markdown": text,
                },
                indent=2,
                ensure_ascii=False,
            )
        )
    else:
        emit(text)
        emit(f"\nwrote {rel}", sys.stderr)
        if rec is None:
            emit("no prep receipt (receipts module unavailable)", sys.stderr)
    return 0


def fit_review(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    key = str(getattr(args, "milestone", "") or "")
    if reg is None or reg.milestones is None:
        emit("the milestone registry did not load; the review cannot be attributed", sys.stderr)
        return 1
    milestone = reg.milestones.get(key)
    if milestone is None:
        known = ", ".join(sorted(reg.milestones.by_key))
        emit(f"unknown milestone {key!r}; known: {known}", sys.stderr)
        return 2
    verdict = str(getattr(args, "verdict", "") or "").upper()
    if verdict not in VERDICTS:
        emit(f"verdict must be one of {VERDICTS}", sys.stderr)
        return 2
    reviewer = str(getattr(args, "reviewer", None) or MISSING)
    note = str(getattr(args, "note", None) or "").strip() or MISSING
    sheet_arg = getattr(args, "sheet", None)
    sheet_rel = ""

    lines = [
        f"TEST FITNESS: {verdict} - {milestone.id} #{milestone.issue}"
        + (f" {getattr(args, 'mechanism', '')}" if getattr(args, "mechanism", None) else ""),
        f"sheet: {sheet_arg or MISSING}",
        f"reviewer: {reviewer}",
        f"note: {note}",
        f"date: {today()}",
        "A FIT verdict clears compute for the configuration in that sheet only (D22). "
        "A NOT_FIT verdict is not a negative: fix the test design and re-register.",
    ]

    if sheet_arg:
        sheet_path = Path(sheet_arg)
        if not sheet_path.is_absolute():
            sheet_path = paths.repo / sheet_path
        text = read_text(sheet_path)
        if not text:
            emit(f"sheet not found: {sheet_arg}", sys.stderr)
            return 1
        sheet_rel = _rel(sheet_path, paths.repo)
        updated = _replace_verdict(text, verdict, reviewer)
        if updated == text:
            updated = text.rstrip("\n") + f"\n- review: TEST FITNESS: {verdict} (reviewer {reviewer}) - {note}\n"
        try:
            sheet_path.write_text(updated, encoding="utf-8")
        except OSError as exc:
            emit(f"cannot update {sheet_rel}: {exc}", sys.stderr)
            return 1
    else:
        note_path = prep_dir(paths) / f"{slug(milestone.key)}-fit-review-{slug(verdict)}-{today()}.md"
        note_path.parent.mkdir(parents=True, exist_ok=True)
        sheet_rel = _rel(note_path, paths.repo)
        note_path.write_text(f"# TEST FITNESS review - {milestone.id}\n\n" + "\n".join(f"- {line}" for line in lines) + "\n", encoding="utf-8")

    _record(
        paths,
        milestone=milestone.key,
        reviewer=reviewer,
        verdict=verdict,
        sheet=sheet_rel,
        note=note,
    )

    body = (
        f"TEST FITNESS: {verdict}\n\n"
        f"- milestone: {milestone.id} #{milestone.issue}\n"
        f"- sheet: {sheet_rel or MISSING}\n"
        f"- reviewer: {reviewer}\n"
        f"- note: {note}\n\n"
        "The pre-registration above states the bar, base, split, steps, stop rule, cost and "
        "minimum test power; this verdict clears compute for that configuration only.\n"
    )
    if getattr(args, "json", False):
        emit(
            json.dumps(
                {
                    "milestone": milestone.key,
                    "issue": milestone.issue,
                    "verdict": verdict,
                    "sheet": sheet_rel,
                    "reviewer": reviewer,
                    "note": note,
                    "comment": body,
                    "post_command": f"gh issue comment {milestone.issue} --body-file -",
                },
                indent=2,
                ensure_ascii=False,
            )
        )
    else:
        emit(f"recorded TEST FITNESS: {verdict} for {milestone.id} (sheet: {sheet_rel})")
        emit("\n--- milestone comment (default: print, never post) ---\n")
        emit(body)
        emit(f"\nTo post it: gh issue comment {milestone.issue} --body-file -   (or add --post)")

    if getattr(args, "post", False):
        from core import run

        rc, out = run(["gh", "issue", "comment", str(milestone.issue), "--body", body], cwd=paths.repo, timeout=30)
        emit(f"gh issue comment {milestone.issue}: exit {rc}")
        if out.strip():
            emit(truncate(out, 300))
        return 0 if rc == 0 else 1
    return 0


def _replace_verdict(text: str, verdict: str, reviewer: str) -> str:
    """Replace the sheet's TEST FITNESS line in place, keeping the rest of the file."""

    def sub(m: re.Match[str]) -> str:
        return f"TEST FITNESS: {verdict} (reviewer {reviewer}, {today()})"

    return _TF_RE.sub(sub, text, count=1)


def list_sheets(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    rows: list[dict[str, str]] = []
    reviews: list[tuple[str, str]] = []
    for f in sheet_files(paths):
        text = read_text(f)
        if not text:
            continue
        verdict = test_fitness(text)
        if verdict == ABSENT:
            # a fit-review note without a sheet line is a review, not a pre-registration
            m = _TF_RE.search(text)
            if "TEST FITNESS review" in text:
                v = re.search(r"TEST FITNESS:\s*(FIT|NOT_FIT)", text, re.IGNORECASE)
                reviews.append((f.name, v.group(1).upper() if v else "?"))
                continue
            rows.append({"sheet": f.name, "milestone": "", "mechanism": "", "verdict": ABSENT, "date": _date_of(f)})
            continue
        rows.append(
            {
                "sheet": f.name,
                "milestone": _field(text, "milestone") or _field(text, "Milestone"),
                "mechanism": _field(text, "mechanism") or _field(text, "Mechanism"),
                "verdict": verdict,
                "date": _field(text, "date") or _date_of(f),
            }
        )
    rows.sort(key=lambda r: (r["date"], r["sheet"]), reverse=True)
    reviews.sort(reverse=True)

    if getattr(args, "json", False):
        emit(json.dumps({"sheets": rows, "reviews": [{"file": n, "verdict": v} for n, v in reviews]}, indent=2))
        return 0
    if not rows and not reviews:
        emit(f"no pre-registration sheets in {_rel(prep_dir(paths), paths.repo)}/")
        return 0
    emit(f"{len(rows)} sheet(s) in {_rel(prep_dir(paths), paths.repo)}/, newest first:")
    for r in rows:
        mark = "  " if r["verdict"] == "FIT" else "! "
        emit(f"{mark}{r['date']:<12} {r['verdict']:<8} {r['milestone'][:28]:<28} {r['mechanism'][:28]:<28} {r['sheet']}")
    if reviews:
        emit(f"\n{len(reviews)} fit-review note(s):")
        for name, verdict in reviews:
            emit(f"  {verdict:<8} {name}")
    emit("\n! = not cleared for compute (D22: a run needs TEST FITNESS: FIT first)")
    return 0


def _date_of(f: Path) -> str:
    m = re.search(r"(\d{4}-\d{2}-\d{2})", f.name)
    return m.group(1) if m else ""


# ------------------------------------------------------------------------------- panels


def _panel_for_milestone(reg: Registry, panel_id: str) -> Panel | None:
    if not panel_id:
        return None
    return find_panel(reg.panels, panel_id)


def _noise_band(noise: str) -> float | None:
    """The numeric half-width of a stated noise, or None when it is unstated.

    Accepts the shapes the repo actually writes: "+-3/40", "±3 rows", "noise 0.01".
    """
    if not noise:
        return None
    m = re.search(r"(?:\+/-|±|\+-)\s*([0-9]*\.?[0-9]+)", noise)
    if m:
        return float(m.group(1))
    m = re.search(r"\bnoise\b[^0-9\-]*([0-9]*\.?[0-9]+)", noise, re.IGNORECASE)
    if m:
        return float(m.group(1))
    return None


def _direction(bar: dict[str, Any]) -> str:
    """'higher' or 'lower' - which way is better for this bar."""
    raw = str(bar.get("direction", "")).strip().lower()
    if raw in ("open", "higher", "up", "floor", "at least", ">="):
        return "higher"
    if raw in ("ceiling", "lower", "down", "cap", "at most", "<=", "must be 0"):
        return "lower"
    current, target = bar.get("current"), bar.get("target")
    try:
        return "higher" if float(target) >= float(current) else "lower"
    except (TypeError, ValueError):
        return "higher"


def _target_met(value: float, current: float, target: float, direction: str) -> bool:
    if direction == "higher":
        return value >= target
    return value <= target


def classify(panel: Panel, bar: dict[str, Any], value: float) -> dict[str, Any]:
    """PASS / FAIL / AT NOISE for one measured value against one frozen bar.

    AT NOISE means the movement is inside the panel's stated noise band, so it must not
    be reported as a real difference - the measured value is still reported.
    """
    current = bar.get("current")
    target = bar.get("target")
    direction = _direction(bar)
    # A bar may state its own noise; otherwise the panel's noise is the only one there is.
    bar_noise = str(bar.get("noise", "") or "").strip()
    noise_text = bar_noise or panel.noise or ""
    band = _noise_band(noise_text)
    out: dict[str, Any] = {
        "panel": panel.id,
        "bar": str(bar.get("id", "?")),
        "value": value,
        "current": current,
        "target": target,
        "direction": direction,
        "noise": noise_text or "unstated",
        "noise_source": f"bar {bar.get('id', '?')}" if bar_noise else ("panel" if panel.noise else "unstated"),
        "noise_band": band,
    }
    try:
        cur = float(current)
        tgt = float(target)
    except (TypeError, ValueError):
        out["outcome"] = "FAIL"
        out["reason"] = "the bar has no numeric current/target, so nothing can be scored against it"
        return out

    met = _target_met(value, cur, tgt, direction)
    improvement = (value - cur) if direction == "higher" else (cur - value)
    needed = (tgt - cur) if direction == "higher" else (cur - tgt)
    out["improvement"] = improvement
    out["needed"] = needed

    if band is not None and needed != 0 and abs(improvement) <= band and improvement != 0:
        out["outcome"] = "AT NOISE"
        out["reason"] = (
            f"moved {improvement:+g} (bar needs {needed:+g}); the stated noise is {noise_text}, "
            "so this is not a real difference"
        )
        return out
    if band is not None and needed != 0 and improvement == 0:
        out["outcome"] = "AT NOISE"
        out["reason"] = f"no movement against a bar that needs {needed:+g}; stated noise {noise_text}"
        return out
    if met:
        out["outcome"] = "PASS"
        out["reason"] = f"{value:g} meets the frozen target {tgt:g} (from {cur:g})"
    elif needed == 0:
        # a forbid bar (target == current, "must be 0"): any movement is a violation,
        # not a difference to measure, so noise does not excuse it
        out["outcome"] = "FAIL"
        out["reason"] = f"{value:g} violates a bar that must stay at {tgt:g} (D23/D24 forbid rule); this is not a noisy improvement"
    else:
        out["outcome"] = "FAIL"
        out["reason"] = f"{value:g} does not reach the frozen target {tgt:g} (from {cur:g}); needs {needed:+g}"

    if band is None and improvement != 0 and needed != 0:
        out["reason"] += " - noise is unstated for this bar, so this cannot be called a real difference (D22 section 3)"
    return out


def show_panel(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    key = str(getattr(args, "milestone", None) or "")
    if reg is None or reg.milestones is None:
        emit("the milestone registry did not load; panels cannot be resolved", sys.stderr)
        return 1
    milestone = reg.milestones.get(key) if key else None
    if key and milestone is None:
        known = ", ".join(sorted(reg.milestones.by_key))
        emit(f"unknown milestone {key!r}; known: {known}", sys.stderr)
        return 2
    if milestone is not None:
        panels = [p for p in reg.panels if p.milestone.lower() == milestone.key or p.id == milestone.panel]
    else:
        panels = list(reg.panels)
    if not panels:
        where = f" for {milestone.id}" if milestone else ""
        emit(f"no panels found{where} in {_rel(paths.panels_dir, paths.repo)}/")
        return 0

    if getattr(args, "json", False):
        emit(
            json.dumps(
                {
                    "panels": [
                        {
                            "id": p.id,
                            "title": p.title,
                            "milestone": p.milestone,
                            "frozen": p.frozen,
                            "grader": p.grader,
                            "source": p.raw.get("source", ""),
                            "noise": p.noise,
                            "noise_band": _noise_band(p.noise),
                            "seeds": p.seeds,
                            "rows": p.rows,
                            "bars": p.bars,
                        }
                        for p in panels
                    ]
                },
                indent=2,
                ensure_ascii=False,
            )
        )
        return 0

    for p in panels:
        emit(f"{p.id} - {p.title}")
        emit(f"  milestone {p.milestone or MISSING}  frozen: {'yes' if p.frozen else 'NO'}  grader: {p.grader or MISSING}")
        emit(f"  source: {p.raw.get('source', MISSING)}")
        emit(f"  noise: {p.noise or MISSING}")
        emit(f"  decisive run: {p.seeds or MISSING}; {p.rows or MISSING}")
        if not p.bars:
            emit(f"  bars: {MISSING}")
        for bar in p.bars:
            of = bar.get("of")
            suffix = f"/{of}" if of not in (None, "") else ""
            unit = bar.get("unit", "")
            emit(
                f"  bar {bar.get('id', '?')}: {bar.get('label', '')} "
                f"{bar.get('current')}{suffix} -> {bar.get('target')}{suffix} {unit} "
                f"[{_direction(bar)} is better]"
            )
        emit("")
    return 0


def score_panel(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    panel_key = str(getattr(args, "panel", "") or "")
    if reg is None or reg.panels is None:
        emit("the panel registry did not load; nothing can be scored", sys.stderr)
        return 1
    panel = find_panel(reg.panels, panel_key)
    if panel is None:
        known = ", ".join(p.id for p in reg.panels) or "none"
        emit(f"unknown panel {panel_key!r}; known: {known}", sys.stderr)
        return 2
    bars = list(panel.bars)
    if not bars:
        emit(f"panel {panel.id} has no bars, so there is nothing to score against", sys.stderr)
        return 1
    bar_key = getattr(args, "bar", None)
    if bar_key:
        bar = next((b for b in bars if str(b.get("id", "")) == bar_key), None)
        if bar is None:
            known = ", ".join(str(b.get("id", "?")) for b in bars)
            emit(f"panel {panel.id} has no bar {bar_key!r}; bars: {known}", sys.stderr)
            return 2
    else:
        # The first bar is the panel's headline bar (the order milestones.json and the
        # panels share), so a score without --bar is not ambiguous. --bar selects any other.
        bar = bars[0]
        if len(bars) > 1:
            emit(
                f"no --bar given: scoring the headline bar {bar.get('id', '?')!r} "
                f"(also available: {', '.join(str(b.get('id', '?')) for b in bars[1:])})",
                sys.stderr,
            )

    value = float(getattr(args, "value"))
    result = classify(panel, bar, value)
    label = getattr(args, "label", None)
    if label:
        result["label"] = str(label)
    if getattr(args, "json", False):
        emit(json.dumps(result, indent=2, ensure_ascii=False))
        return 0
    emit(f"{result['outcome']}  {panel.id} / {result['bar']} = {value:g}")
    emit(f"  {result['reason']}")
    emit(f"  bar {bar.get('label', '')}: {bar.get('current')} -> {bar.get('target')} {bar.get('unit', '')}".rstrip())
    emit(f"  noise used ({result['noise_source']}): {result['noise']}")
    emit(f"  grader: {panel.grader or MISSING}  source: {panel.raw.get('source', MISSING)}")
    if result["outcome"] == "AT NOISE":
        emit("  AT NOISE must never be reported as a real difference.")
    return 0


# -------------------------------------------------------------------------------- rules


#: checkrules.rules() calls every module's register_rules() on every call, so this one
#: must be idempotent: without the cache, two runs in one process produce two rule
#: entries per id and a duplicate RuleResult for every violation.
_RULES_CACHE: list[Rule] | None = None


def register_rules() -> list[Rule]:
    global _RULES_CACHE
    if _RULES_CACHE is None:
        _RULES_CACHE = _build_rules()
    return _RULES_CACHE


def _build_rules() -> list[Rule]:
    return [
        register_rule(
            Rule(
                id="prep.sheet_shape",
                description="every pre-registration sheet carries the required blocks",
                severity="error",
                check=_check_sheet_shape,
                rationale="D22/session-goal clause 4: bar, base, split, steps, stop rule, cost and test power before compute",
            )
        ),
        register_rule(
            Rule(
                id="prep.fit_before_run",
                description="a sheet with a run plan needs TEST FITNESS: FIT before compute",
                severity="warn",
                check=_check_fit_before_run,
                rationale="D22 section 3 and docs/mechanisms/README.md: every mechanism pre-registration gets a FIT review on its milestone before compute",
            )
        ),
        register_rule(
            Rule(
                id="prep.panel_frozen",
                description="every panel referenced by milestones.json exists, is frozen, and carries a grader and a bar",
                severity="error",
                check=_check_panel_frozen,
                rationale="a bar that is not frozen before the run cannot score it; D21/D22 freeze acceptance panels",
            )
        ),
    ]


def _check_sheet_shape(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    out: list[RuleResult] = []
    requirements = sheet_requirements(paths)
    for f in sheet_files(paths):
        text = read_text(f)
        rel = _rel(f, paths.repo)
        if not text.strip():
            out.append(
                RuleResult(
                    rule_id="prep.sheet_shape",
                    severity="error",
                    ok=False,
                    message="empty pre-registration sheet",
                    path=rel,
                    fix=f"rm {rel}",
                )
            )
            continue
        secs = sections(text)
        missing = [name for _key, (name, ok) in requirements.items() if not ok(secs, text)]
        if missing:
            out.append(
                RuleResult(
                    rule_id="prep.sheet_shape",
                    severity="error",
                    ok=False,
                    message=f"missing block(s): {', '.join(missing)}",
                    path=rel,
                    fix="python3 scripts/harness/cli.py prep new --milestone <m> --mechanism <id> "
                    "--run '<run>' --bar '<bar>' --write " + rel,
                )
            )
    return out


def _check_fit_before_run(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    out: list[RuleResult] = []
    for f in sheet_files(paths):
        text = read_text(f)
        if not text.strip():
            continue
        verdict = test_fitness(text)
        if verdict == ABSENT:
            continue
        if verdict in ("FIT",):
            continue
        if not _has_run_plan(text):
            continue
        rel = _rel(f, paths.repo)
        out.append(
            RuleResult(
                rule_id="prep.fit_before_run",
                severity="warn",
                ok=False,
                message=f"TEST FITNESS: {verdict} - not cleared for compute",
                path=rel,
                fix="record the FIT review first: python3 scripts/harness/cli.py prep fit "
                "--milestone <m> --verdict FIT --sheet " + rel + " --reviewer '<name>'",
                meta={"verdict": verdict},
            )
        )
    return out


def _check_panel_frozen(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    out: list[RuleResult] = []
    if reg is None or reg.milestones is None:
        # a registry that cannot load is already an engine-level error; this rule states
        # its own consequence rather than raising out of a check function.
        return [
            RuleResult(
                rule_id="prep.panel_frozen",
                severity="error",
                ok=False,
                message="the milestone registry did not load, so no panel reference can be checked",
                path="docs/harness/milestones.json",
                fix="repair docs/harness/milestones.json (python3 scripts/harness/cli.py check)",
            )
        ]
    seen: list[tuple[str, Any]] = []
    for m in reg.milestones.all():
        if m.panel:
            seen.append((m.panel, m))
    # a panel present in a file but referenced by no milestone is not a violation here.
    # A milestone that has not started has no pre-registered acceptance, so its panel is
    # legitimately unfrozen: freezing is required before the run the panel would score, not
    # before the milestone exists. Those are reported as warnings, never errors.
    not_started_states = {"not-started", "not started", "proposed"}
    for panel_id, milestone in seen:
        rel = f"docs/harness/panels (referenced by {milestone.id} #{milestone.issue} as {panel_id!r})"
        panel = find_panel(reg.panels, panel_id)
        if panel is None:
            out.append(
                RuleResult(
                    rule_id="prep.panel_frozen",
                    severity="error",
                    ok=False,
                    message=f"{milestone.id} names panel {panel_id!r}, which does not exist under docs/harness/panels/",
                    path=rel,
                    fix=f"add docs/harness/panels/{slug(panel_id)}.json with a frozen panel id {panel_id!r}",
                )
            )
            continue
        pending = str(getattr(milestone, "status", "")).lower() in not_started_states
        if pending:
            gaps = []
            if not panel.frozen:
                gaps.append("not marked frozen")
            if not panel.grader:
                gaps.append("no grader")
            if not panel.bars:
                gaps.append("no bars")
            if gaps:
                out.append(
                    RuleResult(
                        rule_id="prep.panel_frozen",
                        severity="warn",
                        ok=False,
                        message=(f"panel {panel.id} ({milestone.id} is {milestone.status}): {', '.join(gaps)}; "
                                 "freeze it before the first run it would score"),
                        path=f"docs/harness/panels/{panel.id}.json",
                        fix=f"set frozen=true, a grader and at least one bar with current and target in docs/harness/panels/{panel.id}.json",
                    )
                )
            continue
        problems: list[str] = []
        if not panel.frozen:
            problems.append("not marked frozen")
        if not panel.grader:
            problems.append("no grader")
        if not panel.bars:
            problems.append("no bars")
        for bar in panel.bars:
            if bar.get("current") in (None, "") or bar.get("target") in (None, ""):
                problems.append(f"bar {bar.get('id', '?')} has no current/target")
        if problems:
            out.append(
                RuleResult(
                    rule_id="prep.panel_frozen",
                    severity="error",
                    ok=False,
                    message=f"panel {panel.id}: {', '.join(problems)}",
                    path=f"docs/harness/panels/{panel.id}.json",
                    fix="freeze the panel: set frozen=true, a grader, and at least one bar with current and target",
                )
            )
    return out


# ------------------------------------------------------------------------------- helpers


def _rel(path: Path, root: Path) -> str:
    try:
        return str(Path(path).resolve().relative_to(Path(root).resolve()))
    except (ValueError, OSError):
        return str(path)


def main(args: Any, paths: HarnessPaths, reg: Registry) -> int:  # pragma: no cover - dispatcher parity
    """cli.py dispatches each prep action to its own function; this is a safe default."""
    action = str(getattr(args, "action", "") or "")
    if action == "new":
        return new_sheet(args, paths, reg)
    if action == "fit":
        return fit_review(args, paths, reg)
    return list_sheets(args, paths, reg)
