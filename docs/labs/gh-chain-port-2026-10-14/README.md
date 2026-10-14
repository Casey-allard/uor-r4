# The `gh` chain — and the pipe deadlock the live test found

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the last piece of `status.registry_vs_github`

## What landed

**`github_issues`** with its cache and `gh` calls: `cache_load`, `cache_save`, `_fresh`,
`trim_comments`, `gh_json`, and the per-issue fetch loop with the four `--json` fields.

**Verified end to end against the live `gh`** — a `#[ignore]`d test that fetches issue #2029 with
comments and asserts the shape: five comments kept, **exactly the three trimmed keys**, and the fetch
written to the cache.

**Coverage stays at 51.** See below — that is deliberate and it is not an oversight.

## The bug the live test found, which no unit test could

**My first `gh_json` polled `try_wait()` and then called `wait_with_output()`.**

**That deadlocks on any output larger than a pipe buffer.** `try_wait` polls the child, but nothing
drains stdout — so once `gh` fills the ~64 KB pipe it **blocks writing and never exits**, and **the
timeout fired on a call that was working perfectly.**

**`gh issue view --json comments` on an active issue is exactly the size that crosses the buffer**, so
the unit tests could not see it, and the first live run failed with `gh timed out` on a `gh` that
returns in under a second.

**Python's `subprocess.run(timeout=…)` reads stdout and stderr concurrently**, which is why the
original never had this. The port now spawns **two reader threads and then polls for exit**, joining
them before returning so no reader outlives the call. **The live test went from `gh timed out` to
passing in 3.1 seconds.**

**This is the case for a live test that unit tests cannot cover**, and it is the second time in this
port that the defect was in *how* a subprocess was driven rather than in what it computed.

## What is still Python, stated precisely

**`github_issues` returns a snapshot; the rule's DECISION after the reading is not ported.** That is
roughly 60 lines of branches after `reading = _card_reading(...)`:

- `no_card` → an info result;
- a **qualitative** card (yes/no) → agree, or report that the registry states a number and the card
  answers differently;
- `reading["current"] is None` → several further branches;
- the numeric comparison against `_registry_numbers`.

**So `status.registry_vs_github` is NOT registered and coverage stays at 51.** The ported code is
**a tested library with no caller** — reachable from tests and from nothing else — **exactly as
`githubcards` was after the previous unit, and registering it would be the silent downgrade this port
refuses.**

## Verification

| check | result |
|---|---|
| the chain against **live `gh`** (`--ignored live_chain`) | **passes in 3.1s**, five comments, exactly three keys each |
| the fetch is written to the cache | asserted |
| the TTL boundary, both sides | asserted |
| a missing or corrupt cache reads as empty | asserted |
| token-shaped strings are redacted **before they reach disk** | asserted |
| an author is read from `author.login` or `user.login` | asserted |
| `cargo test -p uor-r4-harness` | **388 passed, 1 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **51 implemented** |

## Limitations, stated

- **`check` still delegates; this unit moves no coverage.** Two rules remain,
  `harness.documented_commands` and `status.registry_vs_github` — plus `prlint.parse`.
- **The live test is `#[ignore]`d**, so **CI does not run it** and the deadlock it caught would not
  have been caught by CI either. **It asserts shape rather than content, deliberately** — the comments
  on an issue change, and a content assertion would repeat the "true of the tree when written" mistake
  this port has now made three times.
- **The reader threads are not bounded by the deadline.** The parent kills the child on timeout and
  the readers then see EOF, **but a `gh` that forks a grandchild holding the pipe would keep a reader
  alive past the call.** Python's `subprocess.run` has the same exposure.
- **`cache_save` is skipped when nothing was fetched**, so a read served entirely from cache does not
  rewrite it — **which matches Python's `_cache_save` being called unconditionally only after the loop
  ran.** Worth knowing: a cache-only read also does not refresh `fetched_at`, so entries age out on
  schedule rather than being kept alive by being read.
