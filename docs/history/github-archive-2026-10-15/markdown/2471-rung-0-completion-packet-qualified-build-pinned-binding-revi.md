# #2471 — Rung-0 completion packet: qualified build pinned, binding reviews, check-plan PASS (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T14:27:44Z
- **Updated:** 2026-10-14T14:52:32Z
- **Closed:** 2026-10-14T14:36:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2471
- **Merged:** 2026-10-14T14:36:13Z
- **Merge commit:** e96883b81ce0804d636ce59cda83820c8e6eecd9
- **Base / Head:** main / UOR-Foundation:claude/completion-pins
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this fills

The two fail-closed placeholders in `docs/labs/rung0-completion-release-2026-10-14/`. Claude is the owner-named executor ([#2029 6169732401](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6169732401)).

| Field | Value | Source |
|---|---|---|
| `execution.executable_sha256` (both plans) | `d4fbe26cd29f40d0feefe390180207c0c7f665761440efdf9e6bbff44461ac92` | qualified build of frozen `33564b75e`, unmodified |
| `claude_review.sha256` seed 1 | `e7acb4c9fee8d9523be7aa6e612a5f37b0eafb4199ba0ce7fd4d41798d542bf4` | `seed1-review.json` |
| `claude_review.sha256` seed 2 | `f5f6e8905440f83f689ab469c1007b5f9e412901f73719e20bb6eba96300d132` | `seed2-review.json` |

Nothing else in either plan changed: the diff is exactly these two lines per plan.

## Evidence (pod `vast-55939635`, receipts in `receipts/` with `SHA256SUMS`)

- **Build:**
  - 1×RTX 5090, driver 580.178.04, cc 12.0, Ubuntu 24.04.3; nvcc 12.8.93, Rust 1.97.1.
  - Source fetched by sha `33564b75e…`, clean tree.
  - `cargo build --release --locked -p uor-r4-discrete --features cuda`, `UOR_NVCC_ARCH=compute_120`, 24 s.
  - PTX `931a511b…` equals the pilot pin.
- **`completion-build-identity`** (no CUDA context): source, PTX, geometry, driver, config and lineage equal the plan for **both** seeds (`receipts/completion-build-identity-seed{1,2}.json`).
- **Staging verified by sha256:**
  - TRAIN/DEV tokens and masks plus the tokenizer (from the HF store tars).
  - The mirrored `sheet.md`.
  - Parent checkpoint-1640 `46e8ed5e…`, extracted from the pinned archive (MD5 `fb81eaa3…`). Only the `rung0-pilot-20261014/` tree; **no owner record on the pod**.
  - Run-state directories absent.
- **25/25 admitted finite controls PASS** on this executable and host: Route 11, flock 6, driver 8; about 10.7 s of fixtures, no corpus. Run because the host driver (580) differs from the pilot's (595) and this binary had no prior CUDA runtime.
- **`completion-check-plan`:** `PLAN_METADATA_VALID` on both. Plan sha `81128da9…` (seed 1) and `05fa32cc…` (seed 2); `owner_release: STILL_REQUIRED`.

README open items (a)/(b)/(c)/(e) are closed, the stale "pending"/"Codex's build" text is updated, and `current-state.md` has its entry.

## What it does not do

It releases nothing and writes no owner record (R15). **Training stays HELD** until Casey's owner-personal seed-1 `completion-owner-release` (interactive, `UOR_LAB` unset). Then Claude launches seed 1 only. Seed 2 is a separate later owner decision.

Line: evaluation integrity. Headline: unchanged (R14).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6170854694 — 2026-10-14T14:35:16Z — Casey-allard

**Claude exact-head self-review @ `56ebaefe7`: APPROVE.** Since `b3882a680` the only changes are two merges of main: #2470's current-state entry kept beside mine, and the merged bootstrap claim dropped from `claims.json`. The packet bytes are unchanged: SEED1/SEED2 plans `5dc0243c…`/`2e75f3ee…`, reviews `e7acb4c9…`/`f5f6e890…`. `crates/uor-r4-discrete` is identical to `33564b75e`. The packet is also live-validated: Casey's seed-1 release accepted SEED1_PLAN (`plan_sha256 81128da9…`), and seed 1 is running on `vast-55939635` since 14:33:45Z. Queueing.


---
_Comment/review ids in this file: 6170854694_

