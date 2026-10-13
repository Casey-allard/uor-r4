#!/usr/bin/env python3
"""uor-harness: the research harness for UOR-R4.

Zero dependencies (standard library only). Everything this prints is a view over
authoritative repo files and the GitHub issues; nothing here is a new authority.

    python3 scripts/harness/cli.py status
    python3 scripts/harness/cli.py brief --milestone m1
    python3 scripts/harness/cli.py check

See docs/harness/README.md for the full surface and docs/harness/CONTRACT.md for
the module contract.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from core import HarnessPaths, emit  # noqa: E402
from registries import Registry  # noqa: E402

EXIT_OK = 0
EXIT_FAIL = 1
EXIT_USAGE = 2


def _registry(paths: HarnessPaths) -> Registry:
    return Registry.load(paths)


def _add_common(p: argparse.ArgumentParser) -> None:
    p.add_argument("--repo", default=None, help="repo root (default: discovered from cwd)")
    p.add_argument("--json", action="store_true", help="machine-readable output")
    p.add_argument("--quiet", action="store_true", help="suppress notes on stderr")


def _stub(name: str):
    def run(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
        emit(f"uor-harness {name}: module not available yet (scripts/harness/{name}.py)", sys.stderr)
        return EXIT_USAGE

    return run


def _resolve_milestone(reg: Registry, key: str | None, *, required: bool = False):
    """Resolve a milestone key, failing loudly instead of silently doing the wrong thing.

    Silently degrading `--milestone typo` to "show all milestones" hides a user error and
    hands a session a brief for the wrong milestone, which is worse than an error.
    """
    if not key:
        if required:
            raise KeyError("a milestone is required (for example -m m1)")
        return None, []
    ms = reg.milestones.get(key)
    if ms is None:
        known = ", ".join(f"{m.key} ({m.id} #{m.issue})" for m in reg.milestones.all())
        raise KeyError(f"unknown milestone {key!r}; known: {known}")
    return ms, []


# --------------------------------------------------------------------------- status


def cmd_status(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    from status import build_status, render_status

    data = build_status(paths, reg, refresh=getattr(args, "refresh", False))
    if args.json:
        emit(json.dumps(data, indent=2, ensure_ascii=False))
    else:
        emit(render_status(data))
    return EXIT_OK


# ---------------------------------------------------------------------------- brief


def cmd_brief(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    from briefing import assemble

    _resolve_milestone(reg, args.milestone)
    pack = assemble(
        paths,
        reg,
        milestone=args.milestone,
        files=args.files or (),
        query=args.query or "",
        budget_tokens=args.budget_tokens,
    )
    text = pack.render()
    if args.json:
        emit(json.dumps({"title": pack.title, "tokens": pack.tokens, "sources": [s.__dict__ for s in pack.sources], "dropped": pack.dropped, "markdown": text}, indent=2, ensure_ascii=False))
    else:
        emit(text)
    if args.write:
        out = Path(args.write)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(text + "\n", encoding="utf-8")
        if not args.quiet:
            emit(f"wrote {out}", sys.stderr)
    return EXIT_OK


# ---------------------------------------------------------------------------- skill


def cmd_skill(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    from registries import find_skill, match_skills
    from receipts import record

    if args.action == "list":
        rows = [{"id": s.id, "description": s.description, "milestones": s.milestones, "version": s.version} for s in reg.skills]
        if args.json:
            emit(json.dumps({"skills": rows}, indent=2))
        else:
            emit(f"{len(rows)} skills in docs/harness/skills/\n")
            for r in rows:
                emit(f"  {r['id']:<28} {r['description'][:96]}")
        return EXIT_OK

    if args.action == "show":
        skill = find_skill(reg.skills, args.id) if args.id else None
        if skill is None:
            emit(f"unknown skill {args.id!r}", sys.stderr)
            return EXIT_USAGE
        if args.json:
            emit(json.dumps({"id": skill.id, "name": skill.name, "description": skill.description, "when": skill.when, "applies_to": skill.appliesto, "requires": skill.requires, "outputs": skill.outputs, "forbidden": skill.forbidden, "milestones": skill.milestones, "path": str(skill.path), "body": skill.body}, indent=2))
        else:
            emit(skill.path.read_text(encoding="utf-8"))
        return EXIT_OK

    if args.action == "match":
        query = " ".join(args.query or [])
        hits = match_skills(reg.skills, query) if query else []
        if args.json:
            emit(json.dumps({"query": query, "matches": [{"id": s.id, "why": why} for s, why in hits]}, indent=2))
        else:
            if not hits:
                emit(f"no skill matches {query!r}")
            for s, why in hits:
                emit(f"  {s.id:<28} {why}")
        return EXIT_OK

    if args.action == "use":
        skill = find_skill(reg.skills, args.id) if args.id else None
        if skill is None:
            emit(f"unknown skill {args.id!r}", sys.stderr)
            return EXIT_USAGE
        rec = record(
            paths,
            "skill",
            skill=skill.id,
            milestone=args.milestone or "",
            actor=args.actor or "",
            outcome=args.outcome or "started",
            note=args.note or "",
        )
        if args.json:
            emit(json.dumps(rec, indent=2))
        else:
            emit(f"receipt recorded: skill={skill.id} at {rec['ts']}")
            emit(f"  {skill.path}")
        return EXIT_OK

    return EXIT_USAGE


# ----------------------------------------------------------------------------- team


def cmd_team(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    from registries import find_role
    from receipts import record

    if args.action == "roster":
        if args.json:
            emit(json.dumps({"roles": [r.__dict__ for r in reg.roles]}, indent=2))
        else:
            emit(f"{len(reg.roles)} dispatchable roles (docs/harness/roles.json)\n")
            for r in reg.roles:
                ro = " [read-only]" if r.read_only else ""
                emit(f"  {r.id:<24} {r.seat:<11} {r.route:<32} {r.title}{ro}")
                emit(f"      {r.purpose[:100]}")
        return EXIT_OK

    if args.action == "dispatch":
        role = find_role(reg.roles, args.role) if args.role else None
        if role is None:
            emit(f"unknown role {args.role!r}; try: uor-harness team roster", sys.stderr)
            return EXIT_USAGE
        rec = record(
            paths,
            "team",
            role=role.id,
            route=role.route,
            milestone=args.milestone or "",
            task=args.task or "",
            scope=list(args.scope or []),
            count=args.count or 1,
            state=args.state or "dispatched",
        )
        if args.json:
            emit(json.dumps(rec, indent=2))
        else:
            emit(f"dispatched {role.id} -> {role.route} (count {rec['count']}) at {rec['ts']}")
            if rec.get("task"):
                emit(f"  task: {rec['task']}")
            if rec.get("scope"):
                emit(f"  write scope: {', '.join(rec['scope'])}")
            emit(f"  receipt: docs/harness/ledger/{rec['ts'][:7]}.jsonl")
        return EXIT_OK

    return EXIT_USAGE


# ------------------------------------------------------------------------- mistakes


def cmd_mistakes(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    import mistakes as M

    if args.action == "list":
        rows = M.render_index(reg.mistakes)
        if args.json:
            emit(json.dumps({"mistakes": [{"id": m.id, "severity": m.severity, "title": m.title, "guard": m.guard, "class": m.class_} for m in reg.mistakes]}, indent=2))
        else:
            emit(rows)
        return EXIT_OK

    if args.action == "find":
        hmm = M.render_matches(reg, files=args.files or (), query=args.query or "", milestone=args.milestone)
        if args.json:
            emit(json.dumps({"matches": hmm}, indent=2))
        else:
            emit(M.format_matches(hmm))
        return EXIT_OK

    if args.action == "show":
        from registries import find_mistake

        m = find_mistake(reg.mistakes, args.id) if args.id else None
        if m is None:
            emit(f"unknown mistake {args.id!r}", sys.stderr)
            return EXIT_USAGE
        emit(m.path.read_text(encoding="utf-8"))
        return EXIT_OK

    if args.action == "capture":
        return M.capture(args, paths, reg)

    return EXIT_USAGE


# ----------------------------------------------------------------------------- prep


def cmd_prep(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    import prep as P

    if args.action == "new":
        return P.new_sheet(args, paths, reg)
    if args.action == "fit":
        return P.fit_review(args, paths, reg)
    if args.action == "list":
        return P.list_sheets(args, paths, reg)
    return EXIT_USAGE


# ---------------------------------------------------------------------------- panel


def cmd_panel(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    import prep as P

    if args.action == "show":
        return P.show_panel(args, paths, reg)
    if args.action == "score":
        return P.score_panel(args, paths, reg)
    return EXIT_USAGE


# --------------------------------------------------------------------------- budget


def cmd_budget(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    import budget as B

    if args.action == "scan":
        return B.scan(args, paths, reg)
    return B.show(args, paths, reg)


# ----------------------------------------------------------------------------- lint


def cmd_lint(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    from prlint import check_pr, format_pr_body, parse_pr_body

    body = ""
    if args.body_file:
        body = Path(args.body_file).read_text(encoding="utf-8")
    elif args.body:
        body = args.body
    else:
        emit("provide --body or --body-file", sys.stderr)
        return EXIT_USAGE

    if args.action == "format":
        emit(format_pr_body(parse_pr_body(body)))
        return EXIT_OK

    result = check_pr(body, paths, reg)
    if args.json:
        emit(json.dumps(result, indent=2))
    else:
        emit(result["summary"])
        for problem in result["problems"]:
            emit(f"  [{problem['severity']}] {problem['rule']}: {problem['message']}")
            if problem.get("fix"):
                emit(f"      fix: {problem['fix']}")
    return EXIT_OK if result["ok"] else EXIT_FAIL


# ------------------------------------------------------------------------- heartbeat


def cmd_heartbeat(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    import status as S

    return S.heartbeat(args, paths, reg)


# ------------------------------------------------------------------------------ git


def cmd_git(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    import githelp as G

    if args.action == "install-hooks":
        return G.install_hooks(args, paths)
    if args.action == "status":
        return G.status(args, paths)
    return EXIT_USAGE


# ----------------------------------------------------------------------------- check


def cmd_check(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int:
    import checkrules

    # argparse nargs="*" overwrites, so `--only a --only b` would silently drop `a`;
    # flatten the appended lists so every prefix is honoured.
    only: list[str] = []
    raw = getattr(args, "only", None) or []
    for item in raw:
        if isinstance(item, list):
            only.extend(item)
        else:
            only.append(item)
    results = checkrules.run(paths, reg, scope="staged" if args.staged else "repo", only=only or None)
    summary = checkrules.summarize(results)
    if args.json:
        emit(json.dumps({"summary": summary, "results": [r.__dict__ for r in results]}, indent=2, default=str))
    else:
        fails = [r for r in results if not r.ok]
        if not fails:
            emit(f"harness check: {summary['checked']} rules, all pass")
        else:
            emit(f"harness check: {summary['checked']} rules, {summary['failed']} failed "
                 f"({summary['errors']} error, {summary['warnings']} warn)\n")
            for r in fails:
                emit(r.render())
            emit("\nRun `python3 scripts/harness/cli.py check` after each fix; "
                 "`docs/harness/CONTRACT.md` describes every rule.")
    return EXIT_OK if summary["ok"] else EXIT_FAIL


# ------------------------------------------------------------------------------- cli


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="uor-harness",
        description="Research harness for UOR-R4: milestones, skills, expert team, lessons, budgets, PR lint.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="Commands: status, brief, skill, team, mistakes, prep, panel, budget, lint, heartbeat, git, check",
    )
    _add_common(p)
    sub = p.add_subparsers(dest="command")

    sp = sub.add_parser("status", help="what is true now")
    sp.add_argument("--refresh", action="store_true", help="re-read GitHub issues (needs gh)")
    sp.set_defaults(fn=cmd_status)

    sp = sub.add_parser("brief", help="assemble the context pack for a task")
    sp.add_argument("--milestone", "-m", default=None)
    sp.add_argument("--files", "-f", nargs="*", default=())
    sp.add_argument("--query", "-q", default="")
    sp.add_argument("--budget-tokens", type=int, default=14000)
    sp.add_argument("--write", default=None, help="also write the pack to this path")
    sp.set_defaults(fn=cmd_brief)

    sp = sub.add_parser("skill", help="the skill registry")
    sub2 = sp.add_subparsers(dest="action", required=True)
    s = sub2.add_parser("list")
    s.set_defaults(fn=cmd_skill)
    s = sub2.add_parser("show")
    s.add_argument("id", nargs="?")
    s.set_defaults(fn=cmd_skill)
    s = sub2.add_parser("match")
    s.add_argument("query", nargs="*")
    s.set_defaults(fn=cmd_skill)
    s = sub2.add_parser("use", help="record that a skill was actually used")
    s.add_argument("id", nargs="?")
    s.add_argument("--milestone", "-m", default=None)
    s.add_argument("--actor", default=None)
    s.add_argument("--outcome", default="started", choices=["started", "completed", "skipped", "failed"])
    s.add_argument("--note", default=None)
    s.set_defaults(fn=cmd_skill)

    sp = sub.add_parser("team", help="the dispatchable expert team")
    sub2 = sp.add_subparsers(dest="action", required=True)
    s = sub2.add_parser("roster")
    s.set_defaults(fn=cmd_team)
    s = sub2.add_parser("dispatch", help="record a real dispatch and its route")
    s.add_argument("role", nargs="?")
    s.add_argument("--milestone", "-m", default=None)
    s.add_argument("--task", default=None)
    s.add_argument("--scope", nargs="*", default=())
    s.add_argument("--count", type=int, default=1)
    s.add_argument("--state", default="dispatched", choices=["dispatched", "running", "returned", "failed"])
    s.set_defaults(fn=cmd_team)

    sp = sub.add_parser("mistakes", help="durable lessons learned")
    sub2 = sp.add_subparsers(dest="action", required=True)
    s = sub2.add_parser("list")
    s.set_defaults(fn=cmd_mistakes)
    s = sub2.add_parser("find", help="which lessons apply to these files")
    s.add_argument("--files", "-f", nargs="*", default=())
    s.add_argument("--query", "-q", default="")
    s.add_argument("--milestone", "-m", default=None)
    s.set_defaults(fn=cmd_mistakes)
    s = sub2.add_parser("show")
    s.add_argument("id", nargs="?")
    s.set_defaults(fn=cmd_mistakes)
    s = sub2.add_parser("capture", help="write a new lesson and its guard")
    s.add_argument("--id", required=True)
    s.add_argument("--title", required=True)
    s.add_argument("--severity", default="warn", choices=["block", "warn", "info"])
    s.add_argument("--class", dest="class_", default="process")
    s.add_argument("--trigger", nargs="*", default=())
    s.add_argument("--paths", nargs="*", default=())
    s.add_argument("--milestones", nargs="*", default=())
    s.add_argument("--mechanism", default=None)
    s.add_argument("--guard", default=None)
    s.add_argument("--evidence", nargs="*", default=())
    s.add_argument("--symptom", default=None)
    s.add_argument("--lesson", default=None)
    s.add_argument("--force", action="store_true")
    s.set_defaults(fn=cmd_mistakes)

    sp = sub.add_parser("prep", help="pre-registration before compute")
    sub2 = sp.add_subparsers(dest="action", required=True)
    s = sub2.add_parser("new")
    s.add_argument("--milestone", "-m", required=True)
    s.add_argument("--mechanism", default=None)
    s.add_argument("--run", default=None, help="what the run is")
    s.add_argument("--bar", default=None, help="the pre-registered bar")
    s.add_argument("--seeds", type=int, default=None)
    s.add_argument("--rows", type=int, default=None)
    s.add_argument("--base", default=None)
    s.add_argument("--split", default=None)
    s.add_argument("--steps", default=None)
    s.add_argument("--stop", default=None)
    s.add_argument("--cost", default=None)
    s.add_argument("--write", default=None)
    s.set_defaults(fn=cmd_prep)
    s = sub2.add_parser("fit", help="record a TEST FITNESS review")
    s.add_argument("--milestone", "-m", required=True)
    s.add_argument("--verdict", required=True, choices=["FIT", "NOT_FIT"])
    s.add_argument("--sheet", default=None)
    s.add_argument("--reviewer", default=None)
    s.add_argument("--note", default=None)
    s.set_defaults(fn=cmd_prep)
    s = sub2.add_parser("list")
    s.set_defaults(fn=cmd_prep)

    sp = sub.add_parser("panel", help="the frozen acceptance panels")
    sub2 = sp.add_subparsers(dest="action", required=True)
    s = sub2.add_parser("show")
    s.add_argument("--milestone", "-m", default=None)
    s.set_defaults(fn=cmd_panel)
    s = sub2.add_parser("score", help="compare a measured value against the frozen bar")
    s.add_argument("--panel", required=True)
    s.add_argument("--value", type=float, required=True)
    s.add_argument("--bar", default=None)
    s.add_argument("--label", default=None)
    s.set_defaults(fn=cmd_panel)

    sp = sub.add_parser("budget", help="cumulative cost and live compute")
    sub2 = sp.add_subparsers(dest="action")
    s = sub2.add_parser("show")
    s.set_defaults(fn=cmd_budget)
    s = sub2.add_parser("scan", help="pod specs of running jobs (read-only)")
    s.set_defaults(fn=cmd_budget)
    sp.set_defaults(fn=cmd_budget, action="show")

    sp = sub.add_parser("lint", help="check a PR body against the contract")
    sub2 = sp.add_subparsers(dest="action", required=True)
    for name in ("check", "format"):
        s = sub2.add_parser(name)
        s.add_argument("--body", default=None)
        s.add_argument("--body-file", default=None)
        s.set_defaults(fn=cmd_lint)

    sp = sub.add_parser("heartbeat", help="emit the 90-minute status card")
    sp.add_argument("--milestone", "-m", required=True)
    sp.add_argument("--line", default=None)
    sp.add_argument("--count", default=None)
    sp.add_argument("--next", dest="next_step", default=None)
    sp.add_argument("--print", dest="print_card", action="store_true")
    sp.set_defaults(fn=cmd_heartbeat)

    sp = sub.add_parser("git", help="hook installation and delivery status")
    sub2 = sp.add_subparsers(dest="action", required=True)
    s = sub2.add_parser("install-hooks")
    s.set_defaults(fn=cmd_git)
    s = sub2.add_parser("status")
    s.set_defaults(fn=cmd_git)

    sp = sub.add_parser("check", help="run every machine-checkable rule")
    sp.add_argument("--staged", action="store_true", help="only rules that apply to staged files")
    sp.add_argument(
        "--only",
        action="append",
        nargs="+",
        default=None,
        help="rule id prefixes; repeatable, e.g. --only prep --only budget or --only prep budget",
    )
    sp.set_defaults(fn=cmd_check)

    # --json is documented per command; accept it before or after the subcommand so
    # `uor-harness status --json` and `uor-harness --json status` both work.
    for action in sub.choices.values():
        if not any("--json" in a.option_strings for a in action._actions):
            # SUPPRESS, not False: a subparser default would clobber a root-level
            # `--json` placed before the subcommand. SUPPRESS leaves the root value in
            # place when the flag is absent here.
            action.add_argument("--json", action="store_true", default=argparse.SUPPRESS, help=argparse.SUPPRESS)

    return p


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if not getattr(args, "fn", None):
        parser.print_help()
        return EXIT_USAGE
    # `--json` is accepted before or after the subcommand; normalise the tri-state.
    args.json = bool(getattr(args, "json", False))
    try:
        paths = HarnessPaths.discover(repo=args.repo)
    except Exception as exc:  # noqa: BLE001
        emit(f"cannot locate repo root: {exc}", sys.stderr)
        return EXIT_USAGE
    try:
        reg = _registry(paths)
    except (OSError, ValueError, KeyError) as exc:
        emit(f"registry error: {type(exc).__name__}: {exc}", sys.stderr)
        return EXIT_FAIL
    try:
        return int(args.fn(args, paths, reg))
    except BrokenPipeError:  # pragma: no cover
        return EXIT_OK
    except KeyboardInterrupt:  # pragma: no cover
        return 130
    except ImportError as exc:
        emit(f"{args.command}: a harness module is not available yet ({exc}); "
             f"see docs/harness/CONTRACT.md for the module list", sys.stderr)
        return EXIT_USAGE
    except KeyError as exc:
        # a bad milestone or unknown id is a usage error, not an internal failure
        emit(f"{args.command}: {exc.args[0] if exc.args else exc}", sys.stderr)
        return EXIT_USAGE
    except (OSError, ValueError) as exc:
        emit(f"{args.command}: {type(exc).__name__}: {exc}", sys.stderr)
        return EXIT_FAIL


if __name__ == "__main__":
    raise SystemExit(main())
