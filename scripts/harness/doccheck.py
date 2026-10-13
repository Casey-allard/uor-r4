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

#: Docs that give labs commands to run. AGENTS.md is first because it is the one file every lab
#: reads before anything else, so a wrong command there costs the most.
DOCS = (
    "AGENTS.md",
    "docs/harness/README.md",
    "docs/harness/BOOTSTRAP.md",
    "docs/harness/ONBOARD-PROMPT.md",
    "docs/harness/CONTRACT.md",
    "docs/harness/STATE-FORMATS.md",
    "docs/labs/session-goal.md",
)

#: `cli.py` followed by the first token (a command word, or a flag with its value) and the rest of
#: the line, which the tokeniser walks.
_COMMAND_RE = re.compile(
    r"cli\.py\s+"
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


def _resolve(parser: argparse.ArgumentParser, words: list[str], flags: list[str]) -> tuple[bool, str]:
    """Walk the parser tree. Returns (found, explanation).

    Two judgement calls, both so the check tests what matters rather than pedantry:

    * **Global flags may appear anywhere.** argparse only accepts `--repo` before the subcommand, but
      `cli.py check --repo .` reads naturally and works in practice. What is worth checking is that
      the flag EXISTS, not where a doc happened to place it.
    * **Flags interleaved with command words are tolerated**, because
      `cli.py --repo <path> multilab presence` is a valid invocation the naive tokeniser splits.
    """
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
    """Every `cli.py ...` command in one line of documentation. Used by the tests."""
    out: list[dict] = []
    for match in _COMMAND_RE.finditer(text):
        rest = _SUBSTITUTION.sub("$(...)", match.group("rest"))
        tokens = [match.group("first")] + rest.split()
        words, flags = _tokenise(tokens)
        if not words:
            continue
        ok, why = _resolve(parser, words, flags)
        out.append({"command": "cli.py " + " ".join(words + flags), "ok": ok, "problem": why})
    return out


def documented_commands(repo: Path = REPO, parser: argparse.ArgumentParser | None = None) -> list[dict]:
    """Every `cli.py ...` command in the docs, with the file and line it appears on."""
    parser = parser or _build_parser()
    found: list[dict] = []
    seen: set[tuple[str, str]] = set()
    for rel in DOCS:
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

    return [
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
