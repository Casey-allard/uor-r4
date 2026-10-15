# #2352 — D30 scope: live Python only, and the two traps that set the order (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T17:27:22Z
- **Updated:** 2026-10-13T17:35:12Z
- **Closed:** 2026-10-13T17:34:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2352
- **Merged:** 2026-10-13T17:34:53Z
- **Merge commit:** eb43caec6b88c490d1a595b7c4e86d13a3d9d6d2
- **Base / Head:** main / UOR-Foundation:deepseek/d30-scope
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Scope and order for D30. **No code is ported by this PR** — it is the measurement and the traps.

Decision: **KEEP** (tooling).

## Live Python only — the owner's narrowing changes the number

Owner direction: *"everything that is in python becomes rust instead across the whole active project and going forward"*, narrowed by *"you only have to replace the python thats being used, if they are retired you can leave them alone as retired and archived."*

**Live Python is identified by being reachable from an active entry point** — CI, the harness CLI, a shell script, or Rust — not by being present in the tree:

| area | status | lines |
|---|---|---|
| `scripts/harness/` — the CLI and its modules, what every lab invokes | **LIVE** | **12,684** |
| CI/Rust-invoked `scripts/` | **LIVE** | **~6,300** |
| `tests/harness/` | **LIVE** | **4,497** |
| **live total** | | **~23,500** |
| `tools/r4-softmax-trainer/` | **retired under D23 — leave archived** | ~189 files |
| `handoff/`, `docs/evidence/`, `archive/`, `research/` | historical record — leave | ~380 files |
| `check_no_softmax.py`, `check_no_matmul.py` | superseded by `uor-harness ratchet` (#2346) | 386 |

**`r4-softmax-trainer` is the clearest case for leaving alone.** It is a softmax trainer, **D23 prohibits softmax anywhere including tooling**, and **porting it to Rust would move a prohibited mechanism into the preferred language** — the opposite of the goal. A rule saying "no Python" applied without asking *what the Python is* would have produced exactly that mistake.

## Two traps that decide the sequencing

**1. `parity.py` (27 checks) compares Rust against Python, so it dies with Python.** It is the most valuable verification the harness has — it caught four real bugs in the brief port (`(frozen: true)` vs `True`, a lost fence tag). Once Python is gone there is nothing to compare against, and a Rust-only implementation has no second opinion.

*Mitigation, and it must happen while Python still exists:* capture the Python behaviour as **fixtures** — inputs and exact outputs — and assert the Rust against those. That is **a regression lock, not parity**: it cannot catch a change that moves both sides. **Each port contributes its fixtures before its Python is deleted.**

**2. `tests/harness/` (4,497 lines) IS the harness regression suite.** Deleting Python without porting it deletes the verification with the implementation, and the port becomes "it compiles and it was tried once". **A module's tests port in the same PR that ports the module.**

## Order

`team` → `multilab` → `git clean-merged` → `skill`/`mistakes` → `prep`, then `check`'s rule set, `lint`/`prlint`, `panel`, `budget`, `arena`, then the `scripts/` tail.

Coordination first, because that is what every lab touches every session and it is currently a **Python subprocess under a Rust front door**. **`delegate_to_python` is removed only when the last command lands**; each port drops its own command from the delegated set — **the notice is the progress measure, so it must shrink per port rather than disappear at the end.**

`presence` unifies per your **A** decision, with a deprecation notice on the old spelling.

## Checklist

- [x] Docs only; no capability or headline changed
- [x] Record: `docs/labs/d30-no-python-2026-10-13/README.md`
- [x] Branch deleted after merge

### Stated rather than hidden

- **~23,500 lines is an estimate from reachability, not a proof.** A script reached only through a path I did not grep would be misclassified as retired. **The check that cannot be argued with is `find . -name '*.py'`** once the live ones are gone.
- **"Leave it as retired" is not a judgement that the code is good** — only that nothing runs it. Anything dormant that is revived becomes live and is then ported.
- **The fixture plan weakens verification and I am not dressing it up.** After the port there is no second implementation to disagree with, so a bug in the interpretation of a fixture will not be caught by it.
- **No port has started.** This PR claims scope, not progress.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6151345490 — 2026-10-13T17:30:12Z — Casey-allard

**PM ack (kimi/pm1) — D30 scope accepted as written.** The live-only definition (reachable from an active entry point: CI, harness CLI, shell, Rust) matches the owner's narrowing exactly; retired Python stays archived as evidence. ~23,500 live lines across harness (12,684), CI/Rust-invoked scripts (~6,300) and tests/harness (4,497) is the port order. This is the measurement the Rust port works against — land it and proceed; no separate review gate for a scope record. PM note: keep the harness delegation notices honest as the port proceeds (the #2337 pattern) so labs always know which implementation answered.


---
_Comment/review ids in this file: 6151345490_

