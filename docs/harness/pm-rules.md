# Project-manager operating rules

**Kimi lab (session `kimi/pm1`), ratified by the owner 2026-10-13.** These are the rules the PM seat
runs the shared workflow by, ported here so every lab follows the same loop through the same tooling.
The harness instruments each one. The authorities above it are unchanged — the owner,
[DECISIONS.md](../integration/DECISIONS.md), the milestone issues and the dated records stay
authoritative — and these rules only say how work moves between them. Roster and strategic changes
come from the owner through the PM.

## R1 — One owner per branch

- **Rule.** One owner per branch: `team claim` before mutation, `team release` on merge.
- **Why.** Two labs mutating one branch is the one collision git cannot repair.
- **Harness surface.** The committed `docs/harness/claims.json` via `team claim` / `team release`, and
  the `status.branch_claimed` check. A claim is visible repo-wide only once its committed change
  merges — include it in your delivery PR. (Lesson: a local release without the merge is invisible.)

## R2 — One live branch per active PR

- **Rule.** After merge: verify the content on a fresh `origin/main`, delete the branch on both sides,
  remove the worktree, then run `scripts/storage/uor-hygiene --apply`.
- **Why.** Merged branches and idle worktrees are state, not research authority, and they cost disk.
- **Harness surface.** `git clean-merged` proves a squash merge by content, so deletion is safe
  (#2271); `check` reports every orphan as `status.no_orphan_branches`. Owner direction, 9–10 October.

## R3 — Status discipline

- **Rule.** Check in **at least every five minutes** while active: `heartbeat -m <milestone>` (the
  session contract's `--line` and `--count` flags are carried); the card carries headline · count ·
  blocker · next dependency. Presence is live-tier since [#2327](https://github.com/UOR-Foundation/uor-r4/pull/2327):
  the Python `cli.py heartbeat` delegates to the Rust `uor-harness heartbeat`, which is the only
  implementation, and a missing binary is a usage error naming the build command
  (`CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness`).
- **Why.** The board marks a lab **STALE** once it has been quiet longer than the five-minute window —
  stale means working without checking in, which is worse than being visibly absent.
- **Harness surface.** Live-tier presence at `~/.local/share/uor-r4/harness` (no checkout writes);
  `multilab presence` shows the board and says which tier answered.

## R4 — No model compute without a FIT prep sheet

- **Rule.** No model compute without a pre-registration sheet at `TEST FITNESS: FIT` on its milestone.
- **Why.** A bar fixed after the numbers are visible is not a pre-registration (D22).
- **Harness surface.** `prep new` writes the sheet, `prep fit` records the review, and the
  `prep.fit_before_run` check enforces the gate. FIT never lifts an owner hold.

## R5 — Ratchets stay green on every PR

- **Rule.** Every PR keeps the ratchets green: `scripts/check_no_softmax.py` (baseline 1648),
  `scripts/check_no_matmul.py` (93 sites / 26 files), `scripts/check_claim_wording.py`.
- **Why.** The bans erode one "temporary" site at a time; a baseline is a floor, not a budget.
- **Harness surface.** `lint check` invokes the D23 interlock on any PR body that names the operation;
  a PR that raises a baseline fails.

## R6 — Preservation

- **Rule.** Nothing is deleted. Superseded or unmerged source lands as an indexed patch plus an INDEX
  row in `docs/history/branch-archive/`; dated records keep their text.
- **Why.** A deleted artifact cannot be re-examined when a later result depends on it.
- **Harness surface.** `docs/history/branch-archive/INDEX.md` is the index; preservation itself has no
  machine check and is verified in review.

## R7 — Collision resolution

- **Rule.** The PM resolves overlapping claims against the ownership map (AGENTS.md plus the ratified
  splits on the milestone issues); peer source, jobs and leases are never touched; an expired lease is
  suspect, not abandoned — verify liveness before takeover.
- **Why.** A silent takeover is how two labs lose each other's work.
- **Harness surface.** `multilab claims` and `multilab presence` for who is where; the ownership map
  in AGENTS.md; pods stay behind `scripts/pod/uor-pod`.

## R8 — Delivery truthfulness

- **Rule.** An open PR is pending, not merged; partial results say "References #N", never "Closes #N";
  report the exact-head checks actually run; a queue acknowledgement is not a test result.
- **Why.** A delivery claim that outruns the merge state is the one error the record cannot detect
  afterwards.
- **Harness surface.** `lint check` (`prlint.org_rules`, `prlint.checks_at_head`) and the merge state
  on GitHub itself.

## R9 — Roster

- **Rule.** Kimi (PM, `kimi/pm1`); Claude (starling/murmuration read and pods); Codex = Sol (alias,
  same seat: enforcement, M2, head and writer); DeepSeek (harness plane and invariant-addressed
  router/memory); Antigravity (fifth lab, currently offline). Roster changes come from the owner via
  the PM.
- **Why.** Every lab must know who holds a seat before it takes work.
- **Harness surface.** The roster block in `docs/labs/README.md`, cross-checked with AGENTS.md;
  `team roster` lists the dispatchable roles (a separate axis).

## R10 — Exclusive workspaces, and the disk budget that pays for them

- **Rule.** `/Users/casey.allard/uor-r4` (the owner checkout) stays on clean `origin/main`. No lab
  edits, builds, commits, or runs mutating tools in it. **Every lab works in its own isolated
  worktree** — one per active task, claimed under R1 (for example `.worktrees/<lab>-<task>/`).
  Concurrent editing is the permanent condition, so worktrees are required; this is not the
  "a worktree is optional" language of 9 October.

- **Coordination writes never need a checkout.** Since [#2326](https://github.com/UOR-Foundation/uor-r4/pull/2326)
  and [#2327](https://github.com/UOR-Foundation/uor-r4/pull/2327), `heartbeat` publishes presence and
  appends its receipt to the **live tier** (`~/.local/share/uor-r4/harness/`), which every lab on this
  machine shares. `cli.py heartbeat` delegates to `uor-harness heartbeat`, the only implementation.
  `uor-harness ledger-sync` appends your live receipts into the committed shard before you open a PR,
  so the record still reaches review. A check-in therefore dirties nothing, which is the property R10
  exists to protect.

- **The disk budget is part of the rule, not an afterthought.** Each worktree is a full source
  checkout. Measured 13 October: six worktrees cost **6.4 GB** against a **30 GB** floor, and the
  largest driver was **`research/` at 1,002 MB in every checkout — 5,010 MB of the same duplicate** —
  because it was archived to iCloud on 12 October but never untracked. It is now untracked and
  `/research/` is ignored ([#2333](https://github.com/UOR-Foundation/uor-r4/pull/2333)), so a fresh
  worktree carries **no `research/` at all**: ~330–590 MB each instead of ~1.5 GB. **Do not restore it
  inside a worktree.**

- **Cadence, because a rule without one is what filled the disk:**
  - **At merge time** — verify the merge commit on fresh `origin/main`, delete the branch on both
    sides, and **remove your own worktree immediately.** Not "when hygiene next happens".
  - **Before removing it** — anything local-only goes to the archive first: `cloud-store put <lab>
    <dir>` (MD5 round-trip + index), then clear the local copy. Never keep a second local copy of
    archived material.
  - **After each merge** — run `scripts/storage/uor-hygiene --apply` **from the owner checkout**.
  - **`df -h /` after your next delivery**, not on a schedule. Below 30 GB is a stop condition.
  - **Per-worktree build caches are the second sink** (~150–260 MB each): remove the cache with the
    worktree rather than leaving either behind.

- **Why.** 13 October showed the write failure repeatedly — heartbeat/presence landing in another
  session's checkout, the claims store clobbered three times, a ledger shard losing receipts in a
  merge (#2305) — and then the **space** failure: the fix for the first (a worktree per lab) quietly
  multiplied a 1 GB archive nobody had finished untracking. Both are the same mistake in different
  clothes: **a rule written without measuring what it costs.**

- **Harness surface.** `multilab claims` and `multilab presence` show who is where and answer from
  the live tier. `uor-harness shard-check` refuses a ledger shard that loses lines; `ledger-sync`
  carries live receipts into the committed record. `scripts/storage/uor-hygiene --apply` reclaims
  merged worktrees and idle caches.
