# #2289 — Onboarding that verifies instead of asserting: a Rust onboard command (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:39:40Z
- **Updated:** 2026-10-13T04:42:10Z
- **Closed:** 2026-10-13T04:41:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2289
- **Merged:** 2026-10-13T04:41:57Z
- **Merge commit:** c0f9bcc5a6115c2687665a39b503d1cbb42c826a
- **Base / Head:** main / UOR-Foundation:harness/onboard-rust
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Rust tooling, per D27.

## What and why

You asked for something to onboard the labs with, in Rust. This is it.

The install instructions already existed, but they were **prose a lab had to trust**. Nothing told a
lab whether *its own* session satisfied them — and the previous unit shipped two commands that did
not work (a `--dry-run` flag that never existed, and a top-level `install-hooks` that is a `git`
subcommand). A checklist cannot catch either.

## Result

Decision: **KEEP** (tooling).

`uor-harness onboard` **verifies** rather than printing a checklist, and it is **read-only** — no
receipt, no claim, no presence write, no commit. A lab runs it as often as it likes.

```
uor-harness onboard - can this machine use the harness?
  repo:      <repo>
  live tier: ~/.local/share/uor-r4/harness

  ok   python     3.15 found (floor 3.9)
  ok   identity   UOR_LAB=claude
  ok   claims     2 claim(s) visible, 0 held by this lab
  ok   ledger     46 receipts, other labs visible: codex, kimi, lead
  ok   harness    harness check: 4 rules, all pass

READY. Nothing failed. Next:
  - run one real task with the harness: `cli.py brief -m <milestone>` then `cli.py check`
  - claim your branch before you start: `cli.py team claim --branch <name> --lab <lab>`
  - check in every 90 minutes: `cli.py heartbeat -m <milestone>`, which also publishes your presence
  - after your merge, delete your branch on both sides — there is no auto-delete
```

**Exit 0** ready, **3** on failure — distinct from `check`'s exit 1 so automation can tell *"this
machine is not onboarded"* from *"the state drifted"*. Both are failures; they need different fixes.
`--json` for machine-readable output.

### The checks, in the order things actually fail

| check | what a failure means |
|---|---|
| **python** | Python owns behaviour in this port, so no working `python3` means no harness **yet**, however good the Rust binary is |
| **identity** | without `UOR_LAB`, receipts are recorded but attributed to **nobody** |
| **claims** | a lab that cannot read the board is working blind |
| **ledger** | it names the other labs it can see |
| **harness** | the contract every lab runs |

A missing identity is a **FAIL** because everything downstream is worthless without it. An unfamiliar
lab name only **warns** — joining is a decision, not an error.

### A test I wrote wrong found a real bug

My first version warned whenever `UOR_LAB` was not one of the four known names, and I asserted that
`DeepSeek` — capitalised — produced that warning. **The test failed, and the code was wrong, not the
test.** The harness slugs a lab name into every path it touches (`presence/<lab>.json`,
`ledger/<lab>-<month>.jsonl`, the receipt's `lab` field), so `DeepSeek` and `deepseek` are **one lab**.
Warning on capitalisation would have nagged every lab about nothing.

Comparison now happens after slugging, with two tests where there was one: a case variant must be
accepted and slug identically, and an unfamiliar name must warn **without blocking** — because a
nagging check is one people learn to ignore.

```
cargo fmt -p uor-r4-harness -- --check                      clean
cargo clippy -p uor-r4-harness --all-targets -D warnings     0 findings
cargo test -p uor-r4-harness                                29 passed + 1 doc-test
UOR_LAB unset   -> FAIL identity, NOBODY                     exit 3
UOR_LAB=claude  -> READY, names codex/kimi/lead              exit 0
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-onboard-2026-10-13/README.md, plus a current-state entry
- [x] Documented in docs/harness/BOOTSTRAP.md, replacing the hand-verified sequence
- [x] Branch deleted after merge

### Stated rather than hidden

- **It verifies plumbing, not work.** A lab can be fully READY and still misuse the harness.
- **`team dispatch` and `prep new` still have zero foreign receipts.** Onboarding does not change
  that, and `prep new` is the D22 gate standing between a lab and prohibited compute. A lab that did
  not write it should exercise it next — that is the remaining evidence gap.
- **A lab with no Rust toolchain cannot run `onboard`.** It can still use the harness, since Python
  owns behaviour, but it loses the verification step. The doc says so.
- Runtime detection is best-effort and advisory; it never sets identity, because a wrong name in an
  evidence record is worse than a blank.
- `scripts/harness/doccheck.py` — a validator that would have caught the two broken commands
  automatically — is **not in this unit**. It works but still reports false positives, and shipping a
  checker that cries wolf is worse than not having it. Next.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

