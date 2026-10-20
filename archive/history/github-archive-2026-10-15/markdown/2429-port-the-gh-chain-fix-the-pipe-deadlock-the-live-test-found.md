# #2429 — Port the gh chain; fix the pipe deadlock the live test found (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T03:43:58Z
- **Updated:** 2026-10-14T03:47:08Z
- **Closed:** 2026-10-14T03:46:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2429
- **Merged:** 2026-10-14T03:46:44Z
- **Merge commit:** 530b956db5616d6db956cacd550fd337ebc2085d
- **Base / Head:** main / UOR-Foundation:deepseek/gh-chain
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**The `gh` chain — and the pipe deadlock the live test found.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`github_issues`** with its cache and calls: `cache_load`, `cache_save`, `_fresh`, `trim_comments`, `gh_json`, and the per-issue fetch loop with the four `--json` fields.

**Verified end to end against the live `gh`** — an `#[ignore]`d test fetching issue #2029 with comments: five comments, **exactly the three trimmed keys**, and the fetch written to the cache.

**Coverage stays at 51** — deliberately, and the reason is below.

## The bug the live test found, which no unit test could

**My first `gh_json` polled `try_wait()` and then called `wait_with_output()`.**

**That deadlocks on any output larger than a pipe buffer.** `try_wait` polls the child, but **nothing drains stdout** — so once `gh` fills the ~64 KB pipe it **blocks writing and never exits**, and **the timeout fired on a call that was working perfectly.**

**`gh issue view --json comments` on an active issue is exactly that size**, so the unit tests could not see it, and the first live run failed with `gh timed out` on a `gh` that returns in **under a second**.

**Python reads stdout and stderr concurrently while waiting**, which is why the original never had this. The port now spawns **two reader threads and polls for exit**, joining them before returning. **The live test went from `gh timed out` to passing in 3.1 seconds.**

**This is the second time in this port that the defect was in *how* a subprocess is driven rather than in what it computes.**

## What is still Python, stated precisely

**`github_issues` returns a snapshot; the rule's DECISION after the reading is not ported** — roughly 60 lines after `reading = _card_reading(...)`: `no_card`; a **qualitative** card agreeing or disagreeing; `current is None` with its further branches; and the numeric comparison against `_registry_numbers`.

**So the rule is not registered and coverage stays at 51.** The ported code is **a tested library with no caller**, exactly as `githubcards` was after the previous unit — **and registering it would be the silent downgrade this port refuses.**

```
cargo test -p uor-r4-harness -- --ignored live_chain  -> passes in 3.1s against live gh
cargo test -p uor-r4-harness                          -> 388 passed, 1 ignored
cargo fmt --check / clippy -D warnings                -> clean / 0 findings
python3 -m unittest discover -s tests/harness         -> 332 passed
python3 scripts/check_no_softmax.py                   -> OK — 1641, baseline 1641
parity.py                                             -> exit 0, 51 implemented
```

Record: `docs/labs/gh-chain-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates; this unit moves no coverage.** Two rules remain: `harness.documented_commands` and `status.registry_vs_github` — plus `prlint.parse`.
- **The live test is `#[ignore]`d, so CI does not run it** — **and the deadlock it caught would not have been caught by CI either.** It asserts **shape rather than content**, deliberately: an issue's comments change, and a content assertion would repeat the **"true of the tree when written"** mistake this port has now made three times.
- **The reader threads are not bounded by the deadline.** A `gh` that forks a grandchild holding the pipe would keep a reader alive past the call — **Python's equivalent has the same exposure.**
- **`cache_save` is skipped when nothing was fetched**, matching Python. **A cache-only read also does not refresh `fetched_at`**, so entries age out on schedule rather than being kept alive by being read.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

