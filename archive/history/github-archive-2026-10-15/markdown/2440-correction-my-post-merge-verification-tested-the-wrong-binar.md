# #2440 — Correction: my post-merge verification tested the wrong binary (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T04:41:04Z
- **Updated:** 2026-10-14T04:43:46Z
- **Closed:** 2026-10-14T04:43:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2440
- **Merged:** 2026-10-14T04:43:21Z
- **Merge commit:** ad4da4e586a88f3f52e81718fe66a6c65d76a37a
- **Base / Head:** main / UOR-Foundation:deepseek/check-wire-correction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**Correction: my post-merge verification tested the wrong binary.** Tooling only.

Decision: **KEEP** (tooling).

## What I got wrong

**The first check I ran after merging #2438 tested the WRONG BINARY.**

- **`uor-harness` on `PATH`** resolves to **`~/.cargo/bin/uor-harness`**, dated **Oct 13 23:30** — an install predating this work. **It delegated, and it ran 55 rules over Python.**
- **The worktree build** (`local/harness-target/debug/uor-harness`, **Oct 14 00:39**) runs **47 rules over Rust with zero delegation notices.**

**So the merged code is correct and my confirmation of it was not.** This port's own rule — *verify the artifact you shipped, not the one you remember building* — **applies to the binary, not only the source.**

## Why this is a residual risk and not a footnote

**Anyone running `uor-harness check` from `PATH` today still gets the python-delegating binary**, because **nothing in #2438 reinstalls it.**

`docs/harness/BOOTSTRAP.md` says to build with **`CARGO_TARGET_DIR=local/harness-target`**; **it does not say to install — and the install is stale.**

**Not fixed here.** A build-and-install step is a **delivery** question, not part of wiring `check`, and **inventing one silently would be the kind of change this port refuses to make without saying so.**

## Corrected state of #2438's claim

| artifact | `uor-harness check` |
|---|---|
| source on `origin/main` (`591319482`) | handler present, `with_rules` absent, `check` registered |
| **worktree build of that source** | **47 rules, 0 delegation notices** ✅ |
| **`~/.cargo/bin/uor-harness`** | **delegates to Python** ⚠️ |

```
cargo test -p uor-r4-harness                    -> 391 passed, 4 ignored
python3 scripts/harness/parity.py               -> exit 0
./local/harness-target/debug/uor-harness check  -> 47 rules, 0 delegation notices
```

Records updated: `docs/labs/check-wired-2026-10-14/README.md`, `docs/integration/current-state.md`

## Checklist

- [x] The claim in #2438 is right; **the evidence I offered for it named the wrong binary**
- [x] Tooling only; no capability or headline changed
- [x] Correction carried in the record and current state
- [x] Branch deleted after merge

### Stated rather than hidden

- **The stale install is a real, live hazard for every lab** — it is the binary `PATH` finds, and it is the one the harness docs' own command lines invoke.
- **I found this only because I re-ran the check after merging rather than trusting the pre-merge run.** The pre-merge run used the worktree build and was correct; **the failure was in which binary I reached for afterwards.**
- **Three dispatch divergences from the previous unit remain unfixed**, two of which are rules that look like passes because they produce no result at all.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

