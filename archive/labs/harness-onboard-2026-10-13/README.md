# Onboarding that verifies instead of asserting: a Rust `onboard` command

**Question.** The install instructions existed, but they were prose a lab had to trust and follow
correctly. Nothing told a lab whether *its own* session actually satisfied them — and the previous
unit shipped two commands that did not work (a `--dry-run` flag that never existed, and a top-level
`install-hooks` that is a `git` subcommand). A checklist in a document cannot catch either.

**Line:** no model line — tooling. · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

## What it is

`uor-harness onboard` — in the Rust crate, because D27 makes Rust the default for new project code
including tooling. It **verifies** and it is **read-only**: no receipt, no claim, no presence write,
no commit. A lab can run it as many times as it likes.

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

Exit **0** ready, **3** when something FAILs. The 3 is deliberate: distinct from `check`'s exit 1 so
automation can tell *"this machine is not onboarded"* from *"the state drifted"*. Both are failures;
they need different fixes.

## The checks, in the order things actually fail

| check | what a failure means |
|---|---|
| **python** | Python owns behaviour in this port, so a lab with no working `python3` cannot use the harness **yet**, however good the Rust binary is |
| **identity** | without `UOR_LAB`, receipts are recorded but attributed to **nobody** |
| **claims** | a lab that cannot read the board is working blind |
| **ledger** | it names the other labs it can see; an empty list means you are first here, or the shards have not been fetched |
| **harness** | the contract every lab runs |

The identity check is a `FAIL`, not a warning, because everything downstream is worthless without
it. An unfamiliar lab name is a warning instead — joining is a decision, not an error.

## A correctness finding, from a test I wrote wrong

My first version warned whenever `UOR_LAB` was not one of the four known names, and I wrote a test
asserting that `DeepSeek` — capitalised — produced that warning. **The test failed, and the code was
wrong, not the test.**

The Python harness slugs a lab name into every path it touches: `presence/<lab>.json`,
`ledger/<lab>-<month>.jsonl`, and the `lab` field of a receipt. So `DeepSeek` and `deepseek` are
**one lab**, not two. Warning on capitalisation would have nagged every lab about nothing.

The comparison moves after slugging, and there are now two tests where there was one: a case variant
must be accepted and must slug to the same lab, and a genuinely unfamiliar name must warn without
blocking. The second property is the one that matters — a nagging check is a check people learn to
ignore.

## What onboarding deliberately does not do

It does not write a heartbeat, because that would be a state change on a command whose whole value
is being safe to run repeatedly. The doc keeps the by-hand confirmation right after it, clearly
labelled as the step that writes.

It does not detect "your" lab and set `UOR_LAB` for you. It reports what the environment says and,
on a best-effort basis, which runtime it appears to be in — as **advisory only**. Guessing a lab's
identity from the environment would put a wrong name in an evidence record, and a blank is better
than a wrong name.

## Evidence

```
cargo fmt -p uor-r4-harness -- --check                     clean
cargo clippy -p uor-r4-harness --all-targets -D warnings    0 findings
cargo test -p uor-r4-harness                               29 passed + 1 doc-test
```

Behaviour verified in both directions by hand:

```
UOR_LAB unset   -> FAIL identity ... attributed to NOBODY          exit 3
UOR_LAB=claude  -> READY, names codex/kimi/lead as visible         exit 0
```

Read-only verified by test: the receipt count is identical before and after an `onboard` run.

## Limitations

- **It verifies the plumbing, not the work.** A lab can be fully "READY" and still misuse the
  harness. This answers "can you use it", not "are you using it well".
- **`team dispatch` and `prep new` still have zero foreign receipts.** Onboarding does not change
  that; those two paths remain covered by tests and by my own use, which is the weakest evidence
  there is, and `prep new` is the D22 gate standing between a lab and prohibited compute.
- The runtime detection is guesswork by design and says so. It drives a hint, never a decision.
- A lab with no Rust toolchain cannot run `onboard`. It can still use the harness — the Python path
  is what owns behaviour — but it loses the verification step, and the doc says so.
- `scripts/harness/doccheck.py` (a validator for documented commands, which would have caught the
  two broken commands automatically) is **not in this unit**. It works but still reports false
  positives, and shipping a checker that cries wolf is worse than not having it. It is next.
