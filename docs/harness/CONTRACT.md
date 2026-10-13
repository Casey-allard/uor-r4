# Harness interface contract (v1)

Frozen so parallel work cannot collide. `scripts/harness/` is a flat package: modules
import each other directly (`from core import ...`, `from registries import ...`), and
`scripts/harness/cli.py` puts its own directory on `sys.path` before importing.

Standard library only. No network at import time. No model code. Never read secret paths
(use `core.is_secret_path`). Never raise out of a rule function.

## Package layout and ownership

| File | Owns |
|---|---|
| `cli.py` | command dispatch, human + `--json` output |
| `core.py` | paths, front-matter parsing, JSON/JSONL IO, `ContextPack`, redaction |
| `util.py` | small shared helpers (`Result`, `truncate`, markdown section readers) |
| `registries.py` | `Skill`, `Role`, `Mistake`, `Milestone(s)`, `Panel`, `Decision`, `Registry` |
| `checkrules.py` | the rule engine: `Rule`, `RuleResult`, `register`, `run`, registry-level rules |
| `briefing.py` | `assemble()` - the context pack |
| `mistakes.py` | lesson selection and capture; writes `docs/harness/mistakes/` |
| `receipts.py` | skill/team/heartbeat receipts; writes `docs/harness/ledger/` |
| `prep.py` | pre-registration sheet, FIT review, panels |
| `prlint.py` | PR body parse/check/render |
| `status.py` | `status` view and the 90-minute card |
| `budget.py` | cumulative cost, pod specs, disk floor |

## Required module API

Every feature module exposes:

```python
def register_rules() -> list[Rule]: ...          # rules this module owns
def main(args: argparse.Namespace, paths: HarnessPaths, reg: Registry) -> int: ...
```

`main` returns a process exit code (0 ok, 1 verification failure, 2 usage error) and
prints its own output. `--json` output must be a single JSON object on stdout, and must
never include a secret or a raw token.

## Rule engine

```python
@dataclass
class RuleResult:
    rule_id: str
    severity: str          # "error" | "warn" | "info"
    ok: bool
    message: str
    path: str = ""
    fix: str = ""          # the exact command or edit that clears it

@dataclass
class Rule:
    id: str
    description: str
    severity: str
    check: Callable[[HarnessPaths, Registry], list[RuleResult]]
    rationale: str = ""    # owner decision or mistake id
    mistake: str = ""      # the docs/harness/mistakes/<id>.md this rule guards
    scope: str = "repo"    # "repo" | "staged" | "pr"
```

`check` returns only `ok=False` results (one per violation) plus at most one summary
`ok=True`; the engine handles the rest. A rule may persist extra `**meta` on `RuleResult`.

`checkrules.rules()` returns repo-scope rules from every module plus its own.
`checkrules.run(paths, reg, scope="repo", only=None)` returns `list[RuleResult]`.

## Registries

```python
reg = Registry.load(HarnessPaths.discover())      # reads everything once
reg.skills        # list[Skill]      id, name, description, path, body, when[], appliesto[], milestones[]
reg.roles         # list[Role]       id, title, seat, route, purpose, skills[], when[]
reg.mistakes      # list[Mistake]    id, title, severity, trigger[], paths[], milestones[], mechanism, guard, lesson, symptom
reg.milestones    # Milestones.get("m1"|"M1"|"2029"), .all(), .by_lab()
reg.panels        # list[Panel]      id, milestone, frozen, grader, bars[], noise, seeds, rows
reg.decisions     # list[Decision]   id, title, line, body
```

Selection helpers: `match_skills`, `match_mistakes`, `panel_for_milestone`.

## Receipts (`docs/harness/ledger/<YYYY-MM>.jsonl`)

One JSON object per line, sorted keys, `ts` first. Kinds: `skill`, `team`, `heartbeat`,
`context`, `pr`, `prep`. Local ephemeral state (budget/heartbeat cache) goes to
`.harness/` (gitignored). Receipts are the answer to "did the harness actually get used".

```python
receipts.record(paths, kind, **fields) -> dict     # appends and returns the record
receipts.read(paths, month=None, kinds=None) -> list[dict]
receipts.summarize(paths, month=None) -> dict
```

## Context pack

```python
briefing.assemble(paths, reg, *, milestone=None, files=(), query="", budget_tokens=14000) -> ContextPack
```

The pack always states what was loaded, why, and what was left out. `ContextPack.render()`
is what a lab pastes into a session.

## Command signatures (frozen alongside `cli.py`)

Argument names are part of the interface. These are what the dispatcher passes:

```python
briefing.assemble(paths, reg, *, milestone=None, files=(), query="", budget_tokens=14000) -> ContextPack

receipts.record(paths, kind, **fields) -> dict
receipts.read(paths, month=None, kinds=None) -> list[dict]
receipts.summarize(paths, month=None) -> dict

prep.new_sheet(args, paths, reg) -> int      # args: milestone, mechanism, run, bar, seeds, rows,
                                             #       base, split, steps, stop, cost, write, json
prep.fit_review(args, paths, reg) -> int     # args: milestone, verdict, sheet, reviewer, note, json
prep.list_sheets(args, paths, reg) -> int
prep.show_panel(args, paths, reg) -> int     # args: milestone, json
prep.score_panel(args, paths, reg) -> int    # args: panel, value, bar, label, json

budget.show(args, paths, reg) -> int
budget.scan(args, paths, reg) -> int         # read-only: pod specs of running jobs

status.build_status(paths, reg, refresh=False) -> dict
status.render_status(data) -> str
status.heartbeat(args, paths, reg) -> int    # args: milestone, line, count, next_step, print_card, json

githelp.install_hooks(args, paths) -> int
githelp.status(args, paths) -> int           # args: json
```

`prep new` and `heartbeat` must default to **printing** their card and only post to GitHub
when `--post` is passed (add that flag in the module if needed; the Lead will wire it into
`cli.py`). Never post to GitHub from a library function that looks read-only.

## Tests

`tests/harness/` holds one test module per area, plain `unittest`. Run from the repo root:

```
python3 -m unittest discover -s tests/harness -t . -v
```

`tests/__init__.py` and `tests/harness/__init__.py` exist so discovery works with `-t .`.
Tests must not require network, `gh`, or a GPU pod.

## Output conventions

- Human output: short, numbered, with the fixing command. Never a wall of text.
- Always cite the file or issue a fact came from.
- `--json` for machine use.
