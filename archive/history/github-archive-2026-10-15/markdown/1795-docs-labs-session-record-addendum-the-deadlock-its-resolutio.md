# #1795 — docs(labs): session record addendum — the deadlock, its resolution, and the measurement results

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T05:01:02Z
- **Updated:** 2026-10-09T13:22:14Z
- **Closed:** 2026-10-06T05:02:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1795
- **Merged:** 2026-10-06T05:02:06Z
- **Merge commit:** 9f78a15628fa6d360d17e52761324a16546dafc5
- **Base / Head:** main / UOR-Foundation:deepseek/session-record-2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Docs-only. Appends an addendum to `docs/labs/deepseek-session-2026-10-06.md` so its "next dependencies" are not read as current — they are discharged or superseded.

## The coordination deadlock, and its resolution

Records the exact closed cycle:

```
Claim            -> require_current_policy (coord.rs:1398-1401); store pinned 215 commits behind
AdoptPolicy      -> refuses while any attempt is non-finalized (:425)
the one attempt  -> opencode-b0-flock-g1a2-...-g2, reserved against the volume destroyed with the drive
finalize it      -> FinishAttempt needs owned(): a live, unexpired, same-session claim
claim it         -> needs the policy guard to pass -> CYCLE
```

`Action::Abandon` (**#1791**) breaks it by finalizing a `reserved` attempt whose exit evidence **positively proves `never_started`**, requiring **the reserving session**, and deliberately **not** calling `owned()` — that omission is the fix. Applied as `7bc2af58`; preconditions then read 0 live claims, 0 non-finalized attempts. The owner authorised the pointer repair (`e6a40b71`) to **`e445ee946573`** — PR #1758's squash-merge commit. **No policy content changed.** Note `86bec270` (D20 itself) **fails** the guard because `AGENTS.md` changed after it.

Also records the four further rejections, each a real requirement: `work_card` must be literally `sha256:<64-hex>`; **ownership paths must have no trailing slash**; and scope must not overlap a live claim — Codex held one **file** under `examples/`, so claiming the whole **directory** collided.

## Three more runner/host defects

- **A storage WARNING acting as the hard gate** (#1787). The repo says `target 60 | warning 40 | stop 25 GiB`; the local policy used 40 — the warning — as `reserve_bytes`. It blocked the cleanup needed to get back above the goal.
- **`admissions-held.json` had no removal path** (#1793). Holds are now `Terminal | Recheckable` **in the data**, all 13 `host::hold` sites classified (one flagged AMBIGUOUS rather than guessed), and a recheckable hold clears itself by re-running the *same* predicate. **That change alone would not have released the live record** — it predates the `kind` field and reads back Terminal by design; the manual clear was necessary.
- **An orphan audit** (#1790): 3 of 13 DONE dirs unreconciled, **two absent from `coord` entirely**, **one another lab's**.

## Measurement results

- **The matched ordinary control exists now.** `StackArch::Geometric` was hardcoded, so the read had **never** been compared to a parameter-matched ordinary model. Matching is a number: reference **1,370,008**; C6 = 1,370,496 (**+0.036%**) but **+19.9%/+36.0% per-token MACs**; C2 = 1,369,984 (**−0.0018%**) with the access term exact. **Neither is a clean control**, and each residual is stated.
- **The norm-entanglement escape closes** (#1794): on **identical frozen q/k**, cosine **0.0143** vs dot **0.0137** = **1.04×** against a 5× bar. Verdict **PARTIAL** — the pre-registered clauses disagreed and **both readings were reported rather than the flattering one**.
- **Stage-0's frozen-swap was valid only at layer 0** — swapping `config.read` re-runs the forward, so q/k at layers ≥1 are not the vectors A1 scored (**1.85e-07** at layer 0 vs **0.197–0.919** at layers 1–5). **This qualifies how A2b may be cited.**
- **The reframing:** A1's 0.0595 is dominated by the **learned per-distance age bias**. Age zeroed → every family collapses to a flat **1.2–1.7%**; age restored → every family jumps **7.5–13.5× at d16 only**, and **at long range A1 is worse than dot or cosine**. The survivor was a **recency effect**.
- **Named missing component: content identity in the read's q/k projections.** The sieve scores **1.0000** on the same rows; no score function over these vectors finds it.

## Process findings

**An agent's report about shared state is a hypothesis, not a fact** — four checked and wrong. **Subagents are not addressable after dispatch** (two agents duplicated a 12-run matrix because one was thought dead). **Verify the tree, not the PR badge.** **Gate a destructive step on its own check.** And the runner suite has **pre-existing flakes**: 3 of 6 unmodified-baseline runs failed identically.

References #820, #1747, #1787, #1790, #1791, #1793, #1794.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

