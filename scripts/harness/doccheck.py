#!/usr/bin/env python3
"""Check that the commands the docs tell labs to run actually exist.

**Why this exists.** Four documentation errors shipped in three units, and every one was found by
running the thing by hand while writing a record:

1. `heartbeat --dry-run` — no such flag; the real one is `--print`
2. a top-level `install-hooks` — it is a `git` subcommand
3. `team list` — the subcommand is `roster`
4. "`prep new` changes no research state" — it **writes** a sheet

The first three are command-shape errors and this catches them. **The fourth is a claim about
behaviour, and no argument-parser check can catch it** — it is named here so nobody mistakes this for
a completeness guarantee.

Running commands by hand is a poor detector: it only catches a wrong command if the author happens to
try that exact string, and a lab following the doc gets an error without knowing whether the doc or
its own setup is at fault.

This validates every `cli.py <command> <flags>` string in the harness docs against the real argparse
tree. It **parses** rather than **executes**, deliberately: a doc check must be safe against a live
repository, and `heartbeat` or `team claim` would mutate state if actually invoked.

    scripts/harness/doccheck.py            # check, exit 1 on a command that does not exist
    scripts/harness/doccheck.py --json
    scripts/harness/doccheck.py --list     # every command found, and where

Registered as `harness.documented_commands`, so it runs in `check` and therefore in CI.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HARNESS_DIR = Path(__file__).resolve().parent

#: Docs that give labs commands to run, discovered rather than listed.
#:
#: A static list was the first version and it was already incomplete when written: `CONTRIBUTING.md`
#: carries nine `cli.py` commands and was never checked, and any new doc would have been invisible
#: until someone remembered to add it. Discovery means a command cannot go unchecked by omission.
#:
#: Excluded, with reasons:
#:   * `docs/history/` — archived documents, kept verbatim as evidence
#:   * `docs/labs/<topic>-<date>/` — dated records of work already done; their commands describe
#:     what was run at the time, and rewriting them would falsify the record
#:   * `research/` — imported external material
#:   * `.harness/`, `local/`, `target/`, `.worktrees/` — scratch and build space, not documentation
DOC_EXCLUDE_PREFIXES = (
    "docs/history/", "research/", ".harness/", "local/", "target/", ".worktrees/", ".git/",
)

#: Append-only logs that RECORD commands rather than instructing anyone to run them.
#:
#: `current-state.md` is the clearest case: entries there describe what changed, which necessarily
#: includes naming the broken command that was fixed ("a top-level `install-hooks` that is a `git`
#: subcommand"). Checking a changelog against the parser flags the description as if it were
#: guidance, and a check that cries wolf on the project's own log is one people turn off.
DOC_EXCLUDE_FILES = ("docs/integration/current-state.md",)
DOC_EXCLUDE_PARTS = ("/node_modules/",)

#: A dated record directory: `docs/labs/<topic>-<YYYY-MM-DD>/`.
_DATED_RECORD = re.compile(r"^docs/labs/[^/]+-\d{4}-\d{2}-\d{2}/")


def discover_docs(repo: Path = REPO) -> list[str]:
    """Every tracked Markdown file that could give a lab a command, newest concerns first."""
    listing: list[str] = []
    try:
        import subprocess

        out = subprocess.run(
            ["git", "-C", str(repo), "ls-files", "-z", "*.md"],
            capture_output=True, text=True, timeout=30, check=False,
        )
        if out.returncode == 0:
            listing = [p for p in out.stdout.split("\0") if p]
    except (OSError, subprocess.SubprocessError):  # pragma: no cover - git is present in practice
        listing = []
    if not listing:
        # Outside a git checkout, fall back to walking; the exclusions still apply.
        listing = [str(p.relative_to(repo)) for p in repo.rglob("*.md")]
    keep: list[str] = []
    for rel in sorted(listing):
        if any(rel.startswith(pre) for pre in DOC_EXCLUDE_PREFIXES):
            continue
        if any(part in f"/{rel}/" for part in DOC_EXCLUDE_PARTS):
            continue
        if _DATED_RECORD.match(rel) or rel in DOC_EXCLUDE_FILES:
            continue
        if "/" in rel and not rel.startswith("docs/"):
            continue  # only repo-root files and docs/ are documentation
        keep.append(rel)
    # AGENTS.md first: it is the one file every lab reads, so a wrong command there costs the most.
    keep.sort(key=lambda r: (r != "AGENTS.md", r))
    return keep


#: Kept as a named constant for the tests and for anyone reading this expecting a list.
DOCS = ("AGENTS.md",)  # superseded by discover_docs(); see the docstring above

#: The entry point followed by the first token (a command word, or a flag with its value) and the
#: rest of the line, which the tokeniser walks.
#:
#: `uor-harness` is the harness's interface; `cli.py` still matches so a stale invocation is DETECTED
#: by `check` rather than silently passing as unchecked prose. The old form is a rule violation now -
#: see `stale_cli_invocations` - because leaving it in normative docs is what kept four labs typing
#: `python3` for a whole port.
_COMMAND_RE = re.compile(
    r"(?:uor-harness|cli\.py)\s+"
    r"(?P<first>--?[A-Za-z][A-Za-z-]*(?:=\S+)?|[a-z][a-z-]*)"
    r"(?P<rest>[^\n`]*)"
)

#: Flags whose following token is their value, so it must never be read as a command word.
_VALUE_FLAGS = {
    "--repo", "--lab", "--only", "--body-file", "--milestone", "-m", "--next", "--line", "--count",
    "--role", "--branch", "--outcome", "--note", "--files", "-f", "--session", "--mechanism",
    "--run", "--seeds", "--bar", "--sheet", "--task", "--route", "--state", "--verdict",
}

#: Placeholder tokens in docs, not literal arguments.
_PLACEHOLDER = re.compile(r"^<.*>$|^\.\.\.$|^\|$|^…$")

#: A shell command substitution, which is a value and not a harness command. Collapsed to one token
#: before tokenising, because its inner spaces would otherwise split into apparent command words.
_SUBSTITUTION = re.compile(r"\$\([^)]*\)|`[^`]*`")

#: Words the naive tokeniser picks up from a shell substitution or surrounding prose rather than from
#: a harness command (`--repo "$(git rev-parse ...)"`, `export UOR_LAB=kimi`).
_NOT_A_COMMAND = {
    "rev-parse", "show-toplevel", "export", "cd", "echo", "cat", "cp", "then", "and", "or",
    "tee", "sed", "grep", "awk", "xargs", "for", "while", "if", "do", "done", "env", "set",
}


def _build_parser() -> argparse.ArgumentParser:
    """The real argument parser, loaded without running the CLI."""
    spec = importlib.util.spec_from_file_location("_harness_cli_for_doccheck", HARNESS_DIR / "cli.py")
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules["_harness_cli_for_doccheck"] = module
    spec.loader.exec_module(module)
    return module.build_parser()


def _subparsers(parser: argparse.ArgumentParser) -> dict[str, argparse.ArgumentParser]:
    for action in parser._actions:  # noqa: SLF001 - argparse exposes no public accessor
        if isinstance(action, argparse._SubParsersAction):  # noqa: SLF001
            return dict(action.choices)
    return {}


def _all_options(parser: argparse.ArgumentParser, deep: bool = True) -> set[str]:
    """Every option string in the parser tree, long and short."""
    opts: set[str] = set()
    for action in parser._actions:  # noqa: SLF001
        opts.update(o for o in action.option_strings if o.startswith("-"))
    if deep:
        for sub in _subparsers(parser).values():
            opts |= _all_options(sub)
    return opts


def _is_value(token: str) -> bool:
    """Whether the token after a value-taking flag is its value rather than the next flag.

    `--repo <repo> -m m1` must not swallow `-m`; `--repo "$(git rev-parse ...)"` must consume the
    substitution but never treat it as a command.
    """
    if token.startswith("-"):
        return False
    return bool(
        _PLACEHOLDER.match(token)
        or token.startswith(('"', "'", "$"))
        or re.fullmatch(r"[\w./:@=+~-]+", token)
    )


#: Commands the RUST entry point implements and Python does not.
#:
#: The checker walks Python's argparse tree, so a doc naming a Rust-only command looked like a
#: broken invocation - `uor-harness onboard` was reported as `unknown subcommand 'onboard'` in six
#: places. The interface is the Rust binary now, so its commands are valid by definition; this set is
#: what keeps the checker measuring the INTERFACE rather than one implementation of it.
RUST_ONLY_COMMANDS = frozenset(
    {
        "onboard",
        "presence",  # Rust `presence`; Python spells it `multilab presence`
        "registry",
        "prep-render",
        "prep-check",
        "shard-check",
        "check-formats",
        "ledger-sync",
        "notes",
        # #2346 moved the D23/D25 ratchets into Rust. This set has to grow with the port or the
        # checker calls a shipped command unknown - it reported `uor-harness ratchet` as "unknown
        # subcommand" in AGENTS.md, DECISIONS.md and the execution policy, which reddened
        # test_doccheck on main. Keeping it current is part of porting, not an afterthought.
        "ratchet",
        "version",
        # The lesson at docs/harness/mistakes/edit-last-overwrites-another-labs-comment.md names
        # `uor-harness comment edit`, and docs/harness/mistakes/** is inside this checker's discovery
        # scope - so without this entry the resolver calls a shipped command unknown and reddens
        # test_doccheck, exactly as `ratchet` did (#2346). A command is not ported until BOTH this set
        # and the Rust ACCEPTED table know it.
        "comment",
    }
)


def _resolve(parser: argparse.ArgumentParser, words: list[str], flags: list[str]) -> tuple[bool, str]:
    """Walk the parser tree. Returns (found, explanation).

    Two judgement calls, both so the check tests what matters rather than pedantry:

    * **Global flags may appear anywhere.** argparse only accepts `--repo` before the subcommand, but
      `cli.py check --repo .` reads naturally and works in practice. What is worth checking is that
      the flag EXISTS, not where a doc happened to place it.
    * **Flags interleaved with command words are tolerated**, because
      `cli.py --repo <path> multilab presence` is a valid invocation the naive tokeniser splits.
    """
    if words and words[0] in RUST_ONLY_COMMANDS:
        return True, ""
    node = parser
    rest = list(words)
    while rest:
        while rest and rest[0].startswith("-"):
            rest.pop(0)
        if not rest:
            break
        head = rest[0]
        subs = _subparsers(node)
        if subs and head in subs:
            node = subs[head]
            rest.pop(0)
            continue
        if subs:
            return False, f"unknown subcommand {head!r} (have: {', '.join(sorted(subs))})"
        break  # a positional argument: fine
    known = _all_options(parser)
    for flag in flags:
        if flag not in known:
            return False, f"unknown flag {flag!r}"
    return True, "ok"


def _tokenise(text: list[str]) -> tuple[list[str], list[str]]:
    """Split a documented command into (command words, flags)."""
    words: list[str] = []
    flags: list[str] = []
    i = 0
    while i < len(text):
        tok = text[i]
        if tok.startswith("-"):
            flag = tok.split("=", 1)[0]
            flags.append(flag)
            if "=" not in tok and flag in _VALUE_FLAGS and i + 1 < len(text) and _is_value(text[i + 1]):
                i += 1
            i += 1
            continue
        # A bare `-` or a token like `-m1` (flag with an attached value) is still a flag.
        if tok.startswith("-") or re.fullmatch(r"-[A-Za-z]\S*", tok):
            flags.append(tok.split("=", 1)[0])
            i += 1
            continue
        if _PLACEHOLDER.match(tok) or tok in _NOT_A_COMMAND:
            break
        if not re.fullmatch(r"[a-z][a-z-]*", tok):
            break
        words.append(tok)
        i += 1
    return words, flags


def commands_in_text(text: str, parser: argparse.ArgumentParser) -> list[dict]:
    """Every harness invocation in one line of documentation. Used by the tests."""
    out: list[dict] = []
    for match in _COMMAND_RE.finditer(text):
        rest = _SUBSTITUTION.sub("$(...)", match.group("rest"))
        tokens = [match.group("first")] + rest.split()
        words, flags = _tokenise(tokens)
        if not words:
            continue
        ok, why = _resolve(parser, words, flags)
        out.append({"command": "uor-harness " + " ".join(words + flags), "ok": ok, "problem": why})
    return out


def documented_commands(repo: Path = REPO, parser: argparse.ArgumentParser | None = None) -> list[dict]:
    """Every harness invocation in the docs, with the file and line it appears on."""
    parser = parser or _build_parser()
    found: list[dict] = []
    seen: set[tuple[str, str]] = set()
    for rel in discover_docs(repo):
        path = repo / rel
        if not path.is_file():
            continue
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            for row in commands_in_text(line, parser):
                key = (row["command"], rel)
                if key in seen:
                    continue
                seen.add(key)
                found.append({"doc": rel, "line": lineno, **row})
    return found


def stale_cli_invocations(repo: Path = REPO) -> list[dict]:
    """Normative docs that still invoke `python3 scripts/harness/cli.py`.

    **`uor-harness` is the interface.** `cli.py` still exists as an implementation, and the Rust
    entry point delegates to it for the commands that are not ported - but a doc that tells a lab to
    run it puts Python back in front of them, which is what the port was for and what it did not
    achieve for eleven units. A rule rather than a note, because "remember to use the new command"
    is exactly the kind of instruction that decays.

    Dated `docs/labs/<topic>-<date>/` records are excluded by `discover_docs`: they are evidence of
    what was run at the time and rewriting them would falsify the record.
    """
    out: list[dict] = []
    for rel in discover_docs(repo):
        path = repo / rel
        if not path.is_file():
            continue
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if "python3 scripts/harness/cli.py" in line:
                out.append({"doc": rel, "line": lineno, "text": line.strip()[:120]})
    return out


def check(json_out: bool = False, repo: Path = REPO) -> int:
    try:
        rows = documented_commands(repo)
    except Exception as exc:  # noqa: BLE001 - reported, never raised at the caller
        print(f"doccheck: could not build the parser: {type(exc).__name__}: {exc}", file=sys.stderr)
        return 2
    bad = [r for r in rows if not r["ok"]]
    if json_out:
        print(json.dumps({"ok": not bad, "checked": len(rows), "problems": bad}, indent=2))
    else:
        print(f"documented commands: {len(rows)} checked against the real argument parser")
        for r in rows:
            if not r["ok"]:
                print(f"  BROKEN {r['doc']}:{r['line']}  {r['command']}")
                print(f"      {r['problem']}")
        if not bad:
            print("  every documented command exists, with the flags it claims")
        else:
            print("  note: a command can be valid and still be described wrongly in prose;")
            print("        this checks the command, not the claims made about it")
    return 1 if bad else 0


def register_rules():  # noqa: ANN201 - returns list[checkrules.Rule]
    from checkrules import Rule, RuleResult

    def _check(paths, reg=None):  # noqa: ANN001
        try:
            rows = documented_commands(paths.repo)
        except Exception as exc:  # noqa: BLE001 - a rule must report, never raise
            return [
                RuleResult(
                    rule_id="harness.documented_commands",
                    severity="warn",
                    ok=False,
                    message=f"could not check documented commands: {type(exc).__name__}: {exc}",
                    path=str(paths.repo),
                )
            ]
        bad = [r for r in rows if not r["ok"]]
        if not bad:
            return [
                RuleResult(
                    rule_id="harness.documented_commands",
                    severity="warn",
                    ok=True,
                    message=f"all {len(rows)} documented commands exist",
                )
            ]
        return [
            RuleResult(
                rule_id="harness.documented_commands",
                severity="warn",
                ok=False,
                message=(
                    f"{len(bad)} documented command(s) do not exist: "
                    + "; ".join(
                        f"{r['doc']}:{r['line']} `{r['command']}` ({r['problem']})" for r in bad[:3]
                    )
                ),
                path=str(paths.repo),
                fix="fix the doc, or add the command — a lab following it gets an error otherwise",
            )
        ]

    def _check_entry_point(paths, reg=None):  # noqa: ANN001
        """No normative doc may invoke the Python entry point."""
        stale = stale_cli_invocations(paths.repo)
        if not stale:
            return [
                RuleResult(
                    rule_id="harness.entry_point",
                    severity="warn",
                    ok=True,
                    message="no normative doc invokes the Python entry point",
                    path=str(paths.repo),
                )
            ]
        return [
            RuleResult(
                rule_id="harness.entry_point",
                severity="error",
                ok=False,
                message=(
                    f"{len(stale)} normative doc line(s) still invoke `python3 scripts/harness/cli.py`: "
                    + "; ".join(f"{r['doc']}:{r['line']}" for r in stale[:4])
                ),
                path=str(paths.repo),
                fix=(
                    "use `uor-harness <command>` - the Rust binary is the interface and delegates to "
                    "Python for commands that are not ported yet. Dated docs/labs/<topic>-<date>/ "
                    "records are exempt: they are evidence of what was run at the time"
                ),
            )
        ]

    return [
        Rule(
            id="harness.entry_point",
            description="normative docs invoke uor-harness, not the Python entry point",
            severity="error",
            check=_check_entry_point,
            rationale=(
                "`uor-harness` is the harness interface; `cli.py` is an implementation detail behind "
                "it. For eleven units the port left Python as the front door, so four labs typed "
                "python3 forty times a day and could not tell whether they were using the port at "
                "all. A rule rather than a note, because 'remember the new command' is exactly the "
                "instruction that decays."
            ),
        ),
        Rule(
            id="harness.documented_commands",
            description="commands named in the docs exist in the argument parser",
            severity="warn",
            check=_check,
            rationale=(
                "three documentation errors shipped in three units and every one was caught by "
                "running the command by hand: a --dry-run flag that never existed, a top-level "
                "install-hooks that is a git subcommand, and `team list` where the subcommand is "
                "`roster`. A lab following a wrong command gets an error and cannot tell whether "
                "the doc or its own setup is at fault"
            ),
        )
    ]


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="Check that documented harness commands exist.")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--list", action="store_true", help="show every command found")
    ap.add_argument("--repo", default=None)
    args = ap.parse_args(argv)
    repo = Path(args.repo).resolve() if args.repo else REPO
    if args.list:
        for r in documented_commands(repo):
            mark = "ok " if r["ok"] else "BAD"
            print(f"  {mark} {r['doc']}:{r['line']}  {r['command']}")
        return 0
    return check(json_out=args.json, repo=repo)


if __name__ == "__main__":
    raise SystemExit(main())
