# #2422 — Port harness.entry_point; decline harness.documented_commands deliberately (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T02:57:50Z
- **Updated:** 2026-10-14T03:00:25Z
- **Closed:** 2026-10-14T02:59:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2422
- **Merged:** 2026-10-14T02:59:57Z
- **Merge commit:** 6a36f9029c973e8d553c143134f48e8e363efe44
- **Base / Head:** main / UOR-Foundation:deepseek/doccheck
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`harness.entry_point` ported, and the half I deliberately did not port.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`harness.entry_point`** — no normative doc may invoke `python3 scripts/harness/cli.py`.

**Verified against Python on the real repo: 927 docs discovered, `AGENTS.md` first, 0 stale invocations** — Python's own numbers. Coverage **49 of 54**; **3 of `check`'s own rules remain**, down from 5.

## And `harness.documented_commands` is deliberately NOT ported

**This is the one place in this port where I decline to port a rule that is portable.**

Python validates a documented command by **building the live `argparse` parser and resolving the documented words against it** — so it checks the **command, its subcommand and its flags.**

**A Rust version without that introspection could only check the command word.** Shipping that under the same rule id would be **a weaker check wearing the name of the real one** — the silent downgrade this port refuses everywhere else.

**The difference matters because the two catch different mistakes.** A command-word check catches `uor-harness statu`; **it does not catch `uor-harness status --refreshh`** — **and the second is what a lab copying from a doc actually hits.**

## The exclusions, each for its own reason

| excluded | why |
|---|---|
| `docs/labs/<topic>-<date>/` | **evidence of what was run at the time; rewriting it would falsify the record** |
| `docs/integration/current-state.md` | **records commands, does not instruct anyone to run them** |
| `docs/history/`, `research/` | retained history |
| `crates/**/README.md` | **a README beside source is not an instruction set** |

**Excluding them stops the rule firing on the project's own record** — and **a rule that fires on history is one people learn to ignore.** Each asserted with a fixture carrying an invocation that must *not* be reported.

## Coverage was under-reporting, and I believed it

Registering this rule moved the count **47 → 49**, not 48.

**`harness.owner_rules_applied` had been written, tested and shipped in an earlier unit but never added to the inventory** — so coverage **under-reported by one while `parity` reported the rule as "still in Python", and I believed it.**

**The rule that exists to stop owner rules being silently omitted was itself silently omitted from the coverage record.** Found only by editing the same list for a different reason.

## And I fixed a divergence I had written up as "not fixed"

`discover_docs` fell back to an **empty list** when git was unavailable, while **its docstring claimed a walk fallback** — so **off git, Rust found nothing and reported success while Python would still scan.**

The fallback is implemented, skips `.git`/`target`/`node_modules`, and is asserted by a fixture **with no `git init` at all**. **I had written the limitation before checking whether it was one.**

```
cargo test -p uor-r4-harness                    -> 374 passed (5 new)
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0, 49 implemented
```

Record: `docs/labs/entry-point-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** Three of its own rules remain (`harness.documented_commands`, `prlint.gates_declared`, `budget.no_new_spending_class`) plus `status.registry_vs_github`.
- **The docs are scanned for one exact string.** A doc invoking `cli.py` through an alias, a variable, or `sh scripts/harness/cli.py` is **not caught** — Python is the same, **and it is why the rule is a backstop rather than a proof.**
- **The walk fallback is bounded by three directory names**, so off-git discovery is slower — **and it is the path no CI takes, which is why it went unimplemented unnoticed.**
- **The success line is a finding-shaped result at `severity="warn"`** — Python's shape, kept, so **a reader counting warnings sees this rule contribute one on every clean run.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

