"""The brief: assemble the context a task actually needs, and say what was left out.

A lab session starts with `uor-harness brief`. The pack states its own provenance, so
"which context did the agent carry?" stops being a guess. Selection is deterministic and
cited: no model chooses what to load, and nothing is loaded silently.
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path
from typing import Any, Iterable

from core import ContextPack, HarnessPaths, emit
from checkrules import Rule, RuleResult, register as register_rule
from registries import Registry, find_decision, match_mistakes, match_skills, panel_for_milestone
from util import first_markdown_section, read_text, truncate

DEFAULT_BUDGET = 14000
SECTION_LIMITS = {
    "mechanism": 2000,
    "session_goal": 5200,
    "charter": 3000,
    "decision": 2600,
    "mistake": 700,
    "prompt": 2600,
}

_cache: dict[str, str] = {}


def _read(path: Path) -> str:
    key = str(path)
    if key not in _cache:
        _cache[key] = read_text(path)
    return _cache[key]


# ------------------------------------------------------------------ section helpers


_CLAUSE = re.compile(r"\d{1,2}\.\s+\*\*")
_BULLET = re.compile(r"(?:^|\s)-\s+\*\*")
_FENCE = re.compile(r"^```", re.MULTILINE)


def _contract_list(session_goal: str) -> str:
    """Pull clauses 1-10 of the session contract out of session-goal.md.

    The source is one flowing paragraph per clause region, with nested bullets inline.
    Reflow it: a line per numbered clause, and a line per nested bullet.
    """
    body = first_markdown_section(session_goal, "The contract")
    if not body:
        return ""
    # the contract ends at the horizontal rule before the lab prompts
    hr = re.search(r"^\s*(-{3,}|\*{3,})\s*$", body, re.MULTILINE)
    if hr:
        body = body[: hr.start()]
    out: list[str] = []
    for para in re.split(r"\n\s*\n", body):
        if para.lstrip().startswith(("|", "#")):
            continue
        for chunk in _split_keep(_CLAUSE, " ".join(para.split())):
            bullet_parts = _split_keep(_BULLET, chunk.strip())
            first = bullet_parts[0].strip()
            if first:
                out.append(first)
            for b in bullet_parts[1:]:
                out.append("   - " + b.strip().lstrip("- ").strip())
    return "\n".join(out).strip()


def _split_keep(pattern: re.Pattern[str], text: str) -> list[str]:
    """Split on a zero-width lookahead, keeping the boundary at the start of each piece."""
    parts: list[str] = []
    last = 0
    for m in pattern.finditer(text):
        if m.start() == 0:
            continue
        parts.append(text[last : m.start()])
        last = m.start()
    parts.append(text[last:])
    return [p for p in parts if p.strip()]


def _lab_prompt(session_goal: str, lab: str) -> str:
    """Extract the ```text fenced prompt for one lab, without needing its heading to parse."""
    if not lab:
        return ""
    lowered = session_goal.lower()
    idx = -1
    for marker in (f"prompt for the {lab} lab", f"prompt for the {lab.capitalize()} lab"):
        idx = lowered.find(marker.lower())
        if idx >= 0:
            break
    if idx < 0:
        return ""
    opens = [m for m in _FENCE.finditer(session_goal, idx)]
    if len(opens) < 2:
        return ""
    return session_goal[opens[0].end() : opens[1].start()].strip()


def _decision_for(reg: Registry, milestone_key: str, title: str = "") -> list[Any]:
    """Decisions that plausibly govern this milestone: referenced rules first."""
    wanted: list[str] = []
    for rule in reg.milestones.owner_rules:
        wanted.append(str(rule.get("id", "")))
    found = []
    seen = set()
    for key in wanted:
        d = find_decision(reg.decisions, key)
        if d and d.id not in seen:
            seen.add(d.id)
            found.append(d)
    # plus any decision whose body names this milestone issue
    m = reg.milestones.get(milestone_key)
    if m:
        for d in reg.decisions:
            if d.id in seen:
                continue
            if f"#{m.issue}" in d.body or m.id in d.body[:200]:
                seen.add(d.id)
                found.append(d)
    return found


# ------------------------------------------------------------------------ assembler


def assemble(
    paths: HarnessPaths,
    reg: Registry,
    *,
    milestone: str | None = None,
    files: Iterable[str] = (),
    query: str = "",
    budget_tokens: int = DEFAULT_BUDGET,
) -> ContextPack:
    """Build the brief. Always returns a pack; overflow is reported, not hidden."""
    ms = reg.milestones.get(milestone) if milestone else None
    title = "UOR-R4 task brief"
    if ms:
        title += f" - {ms.id} #{ms.issue} ({ms.lab} lab)"
    elif query:
        title += f" - {truncate(query, 60)}"
    pack = ContextPack(title=title)

    # 1. where we are -------------------------------------------------------------
    if ms:
        head = [
            f"- Milestone: **{ms.id} #{ms.issue}** - {ms.title}",
            f"- Lab: `{ms.lab}` - status: `{ms.status}`",
            f"- Headline: **{ms.headline_line()}**",
        ]
        for sec in ms.secondary:
            of = f"/{sec['of']}" if sec.get("of") else ""
            head.append(f"- Secondary: {sec['label']} {sec.get('current')}{of} -> target {sec.get('target')}{of}")
        head.append(f"- Lines (D21 count): {ms.line_counts()}")
        head.append(f"- Acceptance panel: `{ms.panel}`")
        head.append(f"- Base artifact: {ms.base_artifact}")
        head.append(f"- Next action on record: {ms.next}")
        pack.add(f"{ms.id} state (from docs/harness/milestones.json)", "\n".join(head), "docs/harness/milestones.json", "milestone contract")
    else:
        lines = ["| Milestone | Lab | Headline | Lines |", "|---|---|---|---|"]
        for m in reg.milestones.all():
            lines.append(f"| {m.id} #{m.issue} | {m.lab} | {m.headline_line()} | {m.line_counts()} |")
        pack.add("The four milestones", "\n".join(lines), "docs/harness/milestones.json", "no milestone given: show all")

    # 2. the rules that bind ------------------------------------------------------
    if reg.milestones.owner_rules:
        rules = [f"- **{r['id']}** - {r['rule']}" for r in reg.milestones.owner_rules]
        pack.add("Owner rules in force", "\n".join(rules), "docs/harness/milestones.json", "always in force")

    # 2b. the charter
    charter = _read(paths.repo / "CHARTER.md")
    if charter:
        pack.add("The charter (CHARTER.md): constraints and workflow", truncate(charter, SECTION_LIMITS["charter"]), "CHARTER.md", "binding on every lab since 19 Oct 2026")

    # 3. the contract -------------------------------------------------------------
    sg = _read(paths.session_goal_md)
    contract = _contract_list(sg)
    if contract:
        pack.add("Session contract (docs/labs/session-goal.md, clauses 1-10)", truncate(contract, SECTION_LIMITS["session_goal"]), "docs/labs/session-goal.md", "binding on every session")
    if ms:
        prompt = _lab_prompt(sg, ms.lab)
        if prompt:
            pack.add(f"The {ms.lab} lab prompt (the FINISH LINE clause)", truncate(prompt, SECTION_LIMITS["prompt"]), "docs/labs/session-goal.md", f"this milestone's lab is {ms.lab}")

    # 4. governance ---------------------------------------------------------------
    for d in _decision_for(reg, milestone or "", ""):
        pack.add(f"Decision {d.id} - {d.title}", truncate(d.body, SECTION_LIMITS["decision"]), f"docs/integration/DECISIONS.md#L{d.line}", f"governs this milestone")

    # 5. the mechanisms -----------------------------------------------------------
    for mech in (ms.mechanisms if ms else []):
        brief = paths.mechanisms_dir / f"{mech}.md"
        text = _read(brief)
        if not text:
            pack.dropped.append((f"docs/mechanisms/{mech}.md", "brief not found"))
            continue
        fair = first_markdown_section(text, "A fair test needs")
        body = truncate(text, SECTION_LIMITS["mechanism"])
        if fair:
            body += f"\n\n### A fair test needs (verbatim)\n\n{truncate(fair, 900)}"
        pack.add(f"Mechanism brief: {mech}", body, f"docs/mechanisms/{mech}.md", "named by the milestone")

    # 6. the panel ----------------------------------------------------------------
    if ms:
        panel = panel_for_milestone(reg.panels, ms)
        if panel:
            rows = [f"- Panel `{panel.id}` - {panel.title} (frozen: {panel.frozen})", f"- Grader: `{panel.grader}`"]
            if panel.noise:
                rows.append(f"- Panel noise: {panel.noise}")
            if panel.seeds:
                rows.append(f"- Seeds: {panel.seeds}")
            if panel.rows:
                rows.append(f"- Rows: {panel.rows}")
            for bar in panel.bars:
                of = f"/{bar['of']}" if bar.get("of") else ""
                rows.append(f"- Bar `{bar.get('id')}`: {bar.get('label')} {bar.get('current')}{of} -> target {bar.get('target')}{of}")
            pack.add("Frozen acceptance panel", "\n".join(rows), f"docs/harness/panels/{panel.id}.json", "scores this milestone")
        else:
            pack.dropped.append((f"panel {ms.panel}", "no panel file found under docs/harness/panels/"))

    # 7. what went wrong here before ---------------------------------------------
    matches = match_mistakes(reg.mistakes, files=files, query=query or (ms.title if ms else ""), milestones=[ms.key] if ms else [])
    if matches:
        blocks = []
        for m, why in matches[:8]:
            blocks.append(f"**[{m.id}] {m.title}** ({m.severity}, why: {why})\n\n- Symptom: {m.symptom}\n- Lesson: {m.lesson}\n- Guard: `{m.guard or 'none'}`\n- Full text: `{_rel(m.path, paths)}`")
        pack.add("Lessons that apply here (docs/harness/mistakes/)", "\n\n".join(blocks), "docs/harness/mistakes/", f"{len(matches)} lessons match these files/topic")

    # 8. the skills ---------------------------------------------------------------
    skill_query = query or (ms.title if ms else "")
    hits = match_skills(reg.skills, skill_query)
    if files:
        hits += [(s, why) for s, why in match_skills(reg.skills, " ".join(files)) if s not in [h[0] for h in hits]]
    if hits:
        rows = [f"- `{s.id}` - {s.description}  \n  (matched on {why}; path `{_rel(s.path, paths)}`)" for s, why in hits[:6]]
        pack.add("Skills for this task (docs/harness/skills/)", "\n".join(rows), "docs/harness/skills/", "routed from the task description and files")
        pack.add("Loading a skill", "Run `python3 scripts/harness/cli.py skill show <id>` for the full protocol, and `skill use <id>` to record that you loaded it. A skill that is loaded but not receipted is invisible to the owner.", "docs/harness/README.md", "visibility")

    # 9. what is being worked on right now ---------------------------------------
    try:
        from receipts import in_last_hours

        recent = in_last_hours(paths, 24)
        if recent:
            rows = [f"- {r.get('ts', '?')} `{r.get('kind')}` " + " ".join(f"{k}={v}" for k, v in r.items() if k not in ("ts", "kind", "schema", "event", "_file")) for r in recent[-6:]]
            pack.add("Receipts in the last 24h (what actually ran)", "\n".join(rows), "docs/harness/ledger/", "orchestration visibility")
    except ImportError:
        pass

    # 10. the task itself ---------------------------------------------------------
    if files:
        rows = [f"- `{f}`" for f in files]
        pack.add("Files in scope for this task", "\n".join(rows), "cli --files", "supplied by the caller")
    if query:
        pack.add("The task as stated", query, "cli --query", "supplied by the caller")

    pack.add(
        "How to use this brief",
        "1. Post a status card on the milestone issue before changing anything.\n"
        "2. Load the skills listed above with `skill use` so the work is visible.\n"
        "3. Pre-register GPU training or evaluation, and any run a milestone claim rests on, with `prep new`, and get `TEST FITNESS: FIT` before running it.\n"
        "4. When done, add one result row to the line's notebook (`docs/research/notebook/<line>.md`), keep a dated record folder only if the run sealed artifacts, update `docs/integration/NOW.md` if the next action changed, then `lint check` the PR body.",
        "docs/harness/README.md",
        "the loop",
    )

    if pack.tokens > budget_tokens:
        pack.dropped.append(
            (
                "budget",
                f"pack is ~{pack.tokens} tokens against a {budget_tokens} budget; "
                "drop the lowest-value section (mechanism detail first) or raise --budget-tokens",
            )
        )
    return pack


def _rel(path: Path, paths: HarnessPaths) -> str:
    try:
        return str(path.resolve().relative_to(paths.repo))
    except (ValueError, OSError):
        return str(path)


# ------------------------------------------------------------------------------- rules


def register_rules() -> list[Rule]:
    return [
        register_rule(
            Rule(
                id="briefing.sources_exist",
                description="every milestone's mechanism briefs and panels resolve to real files",
                severity="warn",
                check=_check_sources,
                rationale="a brief that cites a missing file teaches the wrong thing",
                mistake="stale-context-pointer",
            )
        ),
        register_rule(
            Rule(
                id="briefing.milestone_numbers_agree",
                description="the registry headline agrees with session-goal.md",
                severity="warn",
                check=_check_headline_agreement,
                rationale="two sources of truth for the headline is how a lab works the wrong number",
            )
        ),
        register_rule(
            Rule(
                id="briefing.lines_agree_with_prompts",
                description="every line declared in session-goal.md is in the registry with the same count",
                severity="warn",
                check=_check_line_agreement,
                rationale="a line that exists on GitHub but not in the registry is invisible to the count",
                mistake="headline-number-not-regenerated",
            )
        ),
    ]


def _check_sources(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    out: list[RuleResult] = []
    for m in reg.milestones.all():
        for mech in m.mechanisms:
            if not (paths.mechanisms_dir / f"{mech}.md").exists():
                out.append(
                    RuleResult(
                        rule_id="briefing.sources_exist",
                        severity="warn",
                        ok=False,
                        message=f"{m.id} names mechanism {mech!r} but docs/mechanisms/{mech}.md does not exist",
                        path="docs/harness/milestones.json",
                        fix=f"fix the mechanism id or add docs/mechanisms/{mech}.md",
                    )
                )
        if panel_for_milestone(reg.panels, m) is None:
            out.append(
                RuleResult(
                    rule_id="briefing.sources_exist",
                    severity="warn",
                    ok=False,
                    message=f"{m.id} names panel {m.panel!r} but no panel file defines it",
                    path="docs/harness/milestones.json",
                    fix=f"add {m.panel} to docs/harness/panels/",
                )
            )
    return out


def _check_headline_agreement(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    text = read_text(paths.session_goal_md)
    if not text:
        return []
    out: list[RuleResult] = []
    for m in reg.milestones.all():
        h = m.headline
        if h.get("vacuous"):
            continue
        cur = h.get("current")
        if cur is None:
            continue
        needle = f"{cur}/{h.get('of')}"
        if needle in text:
            continue
        out.append(
            RuleResult(
                rule_id="briefing.milestone_numbers_agree",
                severity="warn",
                ok=False,
                message=f"{m.id}: registry says {needle}, which docs/labs/session-goal.md does not state",
                path="docs/labs/session-goal.md",
                fix="update the registry entry (or the session-goal text) so a lab cannot work the wrong number",
            )
        )
    return out


def _declared_lines(paths: HarnessPaths) -> list[tuple[str, str, int, int]]:
    """Every `Line "<name>" ... count n/m` declared in the live session-goal prompts.

    The prompts are what a lab actually reads at session start, so a line that exists
    only there is a real line whose count the registry cannot see.
    """
    text = read_text(paths.session_goal_md)
    if not text:
        return []
    built: list[tuple[str, str, int, int]] = []
    # A declaration carries `count n/m`. The contract also shows the PR-line *format*
    # (`Line: <name> · count n/3 · headline ...`), which has no count number and must not
    # be mistaken for a declaration.
    declaration = r"Line\s*[\"“](?P<name>[^\"”]+)[\"”][^\n]{0,80}?count\s*(?P<count>\d+)\s*/\s*(?P<of>\d+)"
    for m in re.finditer(declaration, text):
        built.append(("session-goal contract", m.group("name").strip(), int(m.group("count")), int(m.group("of"))))
    for m in re.finditer(r"DIRECTION:[^\n]*?" + declaration, text):
        built.append(("lab prompt DIRECTION", m.group("name").strip(), int(m.group("count")), int(m.group("of"))))
    # de-duplicate by normalised name, keeping the first (contract) form
    seen: dict[str, tuple[str, str, int, int]] = {}
    for where, name, count, of in built:
        seen.setdefault(name.lower(), (where, name, count, of))
    return list(seen.values())


def _check_line_agreement(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    declared = _declared_lines(paths)
    if not declared:
        return []
    registry_lines: dict[str, tuple[str, int]] = {}
    for m in reg.milestones.all():
        for line in m.lines:
            registry_lines[str(line["name"]).strip().lower()] = (m.id, int(line["count"]))
    out: list[RuleResult] = []
    for where, name, count, of in declared:
        hit = registry_lines.get(name.lower())
        if hit is None:
            out.append(
                RuleResult(
                    rule_id="briefing.lines_agree_with_prompts",
                    severity="warn",
                    ok=False,
                    message=f"line {name!r} is declared in session-goal.md ({where}) but absent from the milestone registry",
                    path="docs/harness/milestones.json",
                    fix=f'add {{"name": "{name}", "count": {count}, "of": {of}}} to the owning milestone in docs/harness/milestones.json',
                )
            )
            continue
        mid, registry_count = hit
        if registry_count != count:
            out.append(
                RuleResult(
                    rule_id="briefing.lines_agree_with_prompts",
                    severity="warn",
                    ok=False,
                    message=f"{mid} line {name!r}: registry says {registry_count}/{of}, session-goal.md says {count}/{of}",
                    path="docs/harness/milestones.json",
                    fix="reconcile the registry with the live prompt (the prompt is what the lab reads)",
                )
            )
    return out


def main(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    pack = assemble(
        paths,
        reg,
        milestone=getattr(args, "milestone", None),
        files=getattr(args, "files", ()) or (),
        query=getattr(args, "query", "") or "",
        budget_tokens=getattr(args, "budget_tokens", DEFAULT_BUDGET),
    )
    emit(pack.render())
    return 0
