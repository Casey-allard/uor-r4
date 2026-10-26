# Decision card (D68): release `m1-export-gap-and-toks` (#2839) and three Stage-T-gated discrete items (#2884, #2883, #2877) from `held` to `ready`

Lab: antigravity, antigravity-95. Items: [`m1-export-gap-and-toks`](../../../../roadmap/items/m1-export-gap-and-toks.md) (#2839), [`r-ranking-with-admission-intact`](../../../../roadmap/items/r-ranking-with-admission-intact.md) (#2884), [`r-rank-objective-keeps-cells`](../../../../roadmap/items/r-rank-objective-keeps-cells.md) (#2883), and [`r-finite-relational-head`](../../../../roadmap/items/r-finite-relational-head.md) (#2877). Status: **approved**. The fresh decision review returned APPROVE in round 1.

## Why these four items were held

1. **Step 6 (`m1-export-gap-and-toks`, #2839, Milestone M1):**
   - `antigravity-74` (#3220) obtained non-author `TEST FITNESS: FIT` on `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`, registered bars B1–B4 via D68 decision review (#2839 comment `6398709883`), and deliberately set `m1-export-gap-and-toks` to `status: held` waiting for Step 4 (`m1-bvg-k-stage-t-training`, #2836) to produce the trained Stage T discrete models on two seeds (`../../m1-export-gap-and-toks/antigravity-74/REPORT.md`).
   - `claude-114` (#3246, `../claude-114/DECISION.md`) confirmed that hold while #2836 was still in progress; `claude-115` (#3247) created #3248 and `claude-116` (#3249, `../../m1-d11-artifact-scorer/claude-116/REPORT.md`) built and qualified the detached `d11-scorer` instrument (`docs/labs/m1-d11-artifact-scorer/instrument/`) so Step 6 can score exported `.uor` files against discrete snapshots and measure integer LUT tokens/s across contexts 512, 2048, and 8192 without touching `crates/uor-r4-discrete` or `crates/uor-r4-integer`.
2. **Stage-T-gated discrete items (`r-ranking-with-admission-intact` #2884, `r-rank-objective-keeps-cells` #2883, `r-finite-relational-head` #2877):**
   - `antigravity-68` (#3213, `../../r-ranking-with-admission-intact/antigravity-68/REPORT.md`), `antigravity-67` (#3212, `../../r-rank-objective-keeps-cells/antigravity-67/REPORT.md`), and `antigravity-65` (#3210, `../../r-finite-relational-head/antigravity-65/REPORT.md`) completed their pre-registration sheets and non-author `TEST FITNESS: FIT` reviews, and placed #2884, #2883, and #2877 on `status: held` waiting for Step 4 (`m1-bvg-k-stage-t-training`, #2836) to finish and for `crates/uor-r4-discrete/**` to be free.

## What changed on `origin/main`

1. `m1-bvg-k-stage-t-training` (#2836) is `status: done` on `origin/main` (merged in #3239 as `c695ad21a`), with its two-seed Stage T checkpoints (`t16_frozen_zero_seed{1,2}`, `t16_relation_seed{1,2}`) archived in `caseyallard/uor-r4-store:results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/stage-t-20261025.tar.gz`.
2. `m1-d11-artifact-scorer` (#3248) is `status: done` on `origin/main` (merged in #3249 as `dd89bb1c6`), providing the detached `d11-scorer` binary in `docs/labs/m1-d11-artifact-scorer/instrument/`.
3. All four subsequent `crates/uor-r4-discrete/**` items (#2880, #2847, #3266, and #3267 `m1-within-cell-writer` merged in #3280 as `bd54e2a18`) are `status: done` on `origin/main`, leaving zero open PRs touching `crates/uor-r4-discrete/**` and causing `uor-harness loop next` to return `WAIT` until these held items are released to `ready`.

## Decision (recommended)

1. **`m1-export-gap-and-toks` (#2839):**
   - Set `status: held` -> `status: ready`.
   - Set `depends_on: [m1-d11-exporter-discrete, m1-bvg-k-stage-t-training, m1-d11-artifact-scorer]` (all three `status: done`).
   - Set `scope: [docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-export-gap-and-toks*, docs/labs/m1-export-gap-and-toks/**, docs/loop/tasks/m1-export-gap-and-toks/**, docs/research/notebook/m4-serving.md]`, matching the detached `d11-scorer` workspace built by #3248 (without locking `crates/uor-r4-discrete/**`, `crates/uor-r4-integer/**`, or `docs/mechanisms/lut-serving.md`, which is held by idle PR #3169, following `claude-121`'s D68 precedent on #2820).
2. **`r-ranking-with-admission-intact` (#2884), `r-rank-objective-keeps-cells` (#2883), and `r-finite-relational-head` (#2877):**
   - Set `status: held` -> `status: ready`.
   - Set `depends_on: [m1-a8-full-dose-rerun, m1-bvg-k-stage-t-training]` (both `status: done`).
   - Add `notebook: docs/research/notebook/m1-discrete-attention.md` and include `crates/uor-r4-discrete/**` and `docs/research/notebook/m1-discrete-attention.md` in `scope:` so each item's implementation and GPU qualification unit can modify `crates/uor-r4-discrete/**` and record its notebook row.

## What each outcome unlocks

| Result | Meaning | Decision |
|---|---|---|
| approved | #2839 (M1 Step 6) and #2884/#2883/#2877 become claimable by `uor-harness loop next` | `m1-export-gap-and-toks` (#2839) runs immediately on the archived Stage T models (`#2836`) using `d11-scorer` (`#3248`), unblocking `#2841` and `#2837`, while a second lab can claim the top discrete item in parallel because `#2839` does not lock `crates/uor-r4-discrete/**` |
| refused | All four items remain `held` | `uor-harness loop next` remains stuck in `WAIT` with zero claimable M1 or discrete items |

Nothing here changes a bar, a stop rule, a technical constraint (§3), or the GPU caps.

## The fresh decision review

- **Round 1: APPROVE** (fresh-context `research` subagent `5c358c1a-0ef9-48be-8329-2c629140c93b`, suborder `local/antigravity/antigravity-95/orders/1.md`).
  - Verified the whole-project context across `CHARTER.md`, `PLAN.md`, all four item files, their prep sheets (`m1-export-gap-and-toks-DRAFT-2026-10-23.md`, `r-ranking-with-admission-intact-2026-10-25.md`, `r-rank-objective-keeps-cells-2026-10-25.md`, `r-finite-relational-head-2026-10-25.md`), and their prior task reports (`antigravity-74`, `claude-114`, `claude-116`, `antigravity-68`, `antigravity-67`, `antigravity-65`).
  - Confirmed that `#2836`, `#3248`, and `#3267` are `status: done` on `origin/main` (`bd54e2a18`) and that `crates/uor-r4-discrete/**` is free.
  - Confirmed that scoping `#2839` to `docs/labs/m1-d11-artifact-scorer/instrument/**` and removing `docs/mechanisms/lut-serving.md` (locked by idle `#3169`) is minimal, lawful, and matches `claude-121`'s approved D68 precedent on `#2820`.
  - Four non-blocking citation advisories (unit numbers `antigravity-65/67/68`, `claude-116`, and merge commit hashes `c695ad21a` and `dd89bb1c6`) were applied before commit.
