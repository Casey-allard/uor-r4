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
  exists to protect. **As of 15 October, `receipt`, `skill use`, `prep fit` and `team dispatch` write the
  live shard too**: they used to append straight into whatever checkout the working directory named, so a
  lab whose shell sat in the owner checkout dirtied it with every receipt.

- **The harness refuses to write the shared checkout — R10 is enforced, not just stated.** A committed
  write (`team claim`/`release`, `ledger-sync`, `multilab sync-live`, `multilab migrate-ledger`,
  `prep new`, `mistakes capture`, `ratchet --update-baseline`, `git install-hooks`) targets **the
  caller's own checkout**, and stops with the rule and the fix when that resolves to the shared one:

  ```
  uor-harness: refusing to write the claims store into the shared checkout /Users/casey.allard/uor-r4
  (R10): the owner checkout stays on clean origin/main, and no lab edits or runs mutating tools in it.
  Run this from your own worktree (.worktrees/<lab>-<task>) or pass --repo <that worktree>; a
  coordination write that needs no checkout at all goes to the live tier.
  ```

  Identification is the shape of the clone (the primary worktree of a repository that has linked
  worktrees) **plus a marker that survives cleanup** — `git config uor.sharedCheckout true`, set once in
  the shared checkout, because a hygiene pass can remove every linked worktree and a shape-only rule
  would then read the owner checkout as an ordinary clone. A linked worktree of a marked checkout is
  still a lab's own. Reads — claims, presence, the ledger — are never refused: reading the shared
  checkout is what R10 permits. **If you hit this, the fix is to `cd` to your worktree, not to reset or
  clean the owner checkout.**

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
    `uor-harness git clean-merged --apply` also **releases the claims on the branches it proves
    merged** (in your own checkout and the live mirror), so a dead claim does not sit in
    `claims.json` until the next lab collides with it. That reaches claims whose branch still exists
    locally; a claim whose branch is already gone is **reported** by the same command — never released
    on a guess — with the `team release` line that clears it.
  - **Base moved under your PR? Merge forward, do not rebase.** `git merge origin/main` into your
    delivery branch, resolve the shared append files by their own rules — `docs/harness/claims.json`
    union, `docs/integration/current-state.md` newest-first — and `git push` plainly: no force, no
    lease, and no dependence on a pre-command hook's approval path. The merge queue **squashes**, so
    the merge commit never reaches `main`'s history, and exact-head review stays exact:
    `git diff origin/main..<new-head> -- <delivered paths>` must equal the reviewed diff. Rebase only
    when a force push is genuinely needed (a stacked child branch); see #2519 for that path.
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

## R11 — GitHub access: the local `gh` CLI; rule changes come from the PM

- **Rule.** The locally installed, owner-authenticated `gh` CLI is every lab's standard interface to
  GitHub: issue comments, PR review and merge-queue operations, branch and ruleset checks. The
  harness remains the coordination plane (presence, claims, notes, receipts, prep sheets); `gh` is
  the read/write path for durable GitHub content. Do not hand-roll API clients, scrape the web UI,
  or invent per-lab GitHub tooling when `gh` already answers the question.

- **Rate discipline.** Every lab's `gh` calls share the owner's hourly API budget. Page issue
  comments from the last page, fetch full bodies by comment id, and keep polling at the five-minute
  cadence — `uor-harness notes` is the awareness layer precisely so four labs do not each poll
  every issue. A rate-limit trip is a signal to lean on the harness cache, not to retry harder.

- **Who changes the rules.** R-series coordination rules are written by the PM (Kimi lab) and
  delivered through protected PRs like any other change; D-series decisions are the owner's alone.
  The harness owner (DeepSeek) changes the harness when the PM asks or when it finds a defect
  itself, and reports the change on #2028. No other lab edits `pm-rules.md`.

- **Why.** 13 October: rule text and harness behavior were drifting in from two directions at once,
  and labs were polling GitHub ad hoc until the API rate limit tripped mid-scan. One rule-writer and
  one standard client keep the plane legible.

## R12 — No test bar rules out a mechanism; instrument first, solvability before authority

- **Rule.** No numeric check, tolerance or acceptance bar may rule out, close or redirect a
  mechanism or a work line — ever. A failed check suspends that run's claims only; what the failure
  *means* is decided by analysis with full project context, never by the bar itself. Stopping a
  mechanism remains the owner's decision alone (D22), and a tooling, instrument or numerical failure
  is a blocker to repair, not evidence about the mechanism.

- **Before a check is given authority**, the pre-registration author and the TEST FITNESS reviewer
  must each answer four questions, on the sheet:
  1. What quantity does this test actually measure?
  2. Is that quantity computable at all at the current mechanism stage?
  3. Can the instrument beat the bar in principle? A bar the instrument cannot reach (precision,
     resolution, sample size, dtype noise floor) makes the test invalid, not the mechanism failed.
  4. Should this test run at this stage at all?
  A FIT verdict without these four answers is incomplete; labs may ask them of any sheet at review.

- **Failure attribution order.** Instrument, then fixture, then bar, then mechanism — examined in
  that order, each with evidence. A precision failure is repaired as instrumentation (for example an
  F64 oracle grading an F32 secant check, production path untouched); it never licenses editing the
  mechanism to please the test.

- **Build-first.** While a mechanism is under construction, its checks verify construction
  correctness — exact laws, causality, refusal behavior, interface integrity. Optimization and
  performance bars apply only after the mechanism exists and is wired. Do not let a tuning gate
  block a construction gate.

- **Why.** Owner direction, 13 October 2026: the project has repeatedly been derailed by arbitrary
  test values acquiring veto power — including waking to find entire work lines reverted over bars
  nobody had questioned. The same day, control 11 (`murm_backward_secants`) stopped a 33-control
  qualification line on a 0.002 relative-error bar that an F32 finite-difference secant at h=1e-3
  may be unable to beat in principle. A test whose meaning, solvability and stage-appropriateness
  have not been established has no authority. Every failure gets an external full-context review
  before it counts — and the first question that review answers is whether the test should have been
  run at all.

## R12 — No test bar rules out a mechanism; instrument first, solvability before authority

- **Rule.** No numeric check, tolerance or acceptance bar may rule out, close or redirect a
  mechanism or a work line — ever. A failed check suspends that run's claims only; what the failure
  *means* is decided by analysis with full project context, never by the bar itself. Stopping a
  mechanism remains the owner's decision alone (D22), and a tooling, instrument or numerical failure
  is a blocker to repair, not evidence about the mechanism.

- **Before a check is given authority**, the pre-registration author and the TEST FITNESS reviewer
  must each answer four questions, on the sheet:
  1. What quantity does this test actually measure?
  2. Is that quantity computable at all at the current mechanism stage?
  3. Can the instrument beat the bar in principle? A bar the instrument cannot reach (precision,
     resolution, sample size, dtype noise floor) makes the test invalid, not the mechanism failed.
  4. Should this test run at this stage at all?
  A FIT verdict without these four answers is incomplete; labs may ask them of any sheet at review.

- **Failure attribution order.** Instrument, then fixture, then bar, then mechanism — examined in
  that order, each with evidence. A precision failure is repaired as instrumentation (for example an
  F64 oracle grading an F32 secant check, production path untouched); it never licenses editing the
  mechanism to please the test.

- **Build-first.** While a mechanism is under construction, its checks verify construction
  correctness — exact laws, causality, refusal behavior, interface integrity. Optimization and
  performance bars apply only after the mechanism exists and is wired. Do not let a tuning gate
  block a construction gate.

- **Why.** Owner direction, 13 October 2026: the project has repeatedly been derailed by arbitrary
  test values acquiring veto power — including waking to find entire work lines reverted over bars
  nobody had questioned. The same day, control 11 (`murm_backward_secants`) stopped a 33-control
  qualification line on a 0.002 relative-error bar that an F32 finite-difference secant at h=1e-3
  may be unable to beat in principle. A test whose meaning, solvability and stage-appropriateness
  have not been established has no authority. Every failure gets an external full-context review
  before it counts — and the first question that review answers is whether the test should have been
  run at all.

## R13 — Discrete learning path: rung re-registration and the instrument boundary (implements D32)

**Authority, October 13:** owner delegation in session — "choose what is best for the project
overall in the sense of what our ultimate goal is — a working geometric language model... make
the best choice among the allowed." This rule is the PM's ruling under that delegation, within
D32 and all standing decisions.

- **Ruling 1 — rung 0 re-registers as a new sheet on a compliant discrete learning law.**
  The frozen F32 sheet (`docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md`, FIT at
  Amendment 6, frozen by Amendment 7) remains the design record and correctness baseline. Its
  lawful pins carry over to the successor sheet where they survive the substrate change: the
  pinned panel and data hashes, seed discipline, the count-predictor reference class, anti-vacuity
  guards, the dev evaluation policy and scale-invariant rank measures. The learning law,
  optimizer and instruments are redesigned discrete: admissible-move selection and local credit
  over the geometry-preserving SpiralCore/FBS transport, no floating-point optimizer, no dense
  learned maps as a prerequisite.
- **Ruling 2 — no floating point in instruments that verify the model path.** Qualification of
  discrete kernels uses exact integer/rational arithmetic, stored expected values, analytic exact
  values, or exhaustive enumeration over small discrete state spaces. No floating-point oracle,
  on any device, even test-only. For a discrete law an exact check is always available in
  principle; if a future check appears to genuinely require floating point to evaluate, it does
  not run — it returns to the owner as a named exception request with the reason. Rationale: D32
  is a clean break; an FP oracle would reintroduce the exact ambiguity class it removed, and
  "instrument-only FP" is how the F32/F64 path grew back the last time.
- **Unchanged:** D24's compute placement (harness, docs, source review and grading of stored
  replies on the laptop are not model compute); R12's four authority questions and attribution
  order, applied to the discrete instruments; D31's instrument-defect rule; the frozen-bar
  discipline. A discrete instrument's defect is still repaired as instrumentation, never as a
  mechanism edit.
- **Sequencing:** Codex specifies the discrete learning law for review (D32 item 6); the PM owns
  the successor rung-0 sheet and requests TEST FITNESS from Claude only when the law is pinned
  in source with exact expected-value controls. No compute before FIT, and no FIT releases the
  training hold — the owner does.

## R14 — No target is set or restated off a vacuous instrument

**Authority, October 13:** owner direction in session — a test criterion that is arbitrary is not
legal under the harness; the owner is not to be asked to pick numbers off such criteria.

- **A headline target exists only when its instrument passes R12's four questions on evidence.**
  If an instrument is shown vacuous (a constant answer clears half the panel), confounded
  (a copy control ties the model), or noisy beyond the bar, then every target and every
  pass/fail reading derived from it is **void**, not "approximately informative".
- **Application: the M1 reply-quality headline is suspended.** The qwen-graded 116/232 target is
  banned under D28 (neural grader); the deterministic 88-row re-score is vacuous under Claude's
  F3/F5 audit findings (a constant canned reply scores 40/88; several zeros are instrument false
  negatives). Neither number, nor any re-scaled variant of them, binds anything. There is no M1
  reply target until the v2 check table exists with frozen positive controls, per-category
  best-constant pairing and abstention-as-discrimination scoring — and even then a target is set
  from measured mechanism behavior, not picked.
- **Build-first governs (R12).** While the discrete mechanism is under construction, what binds
  is construction correctness: exact laws, exact controls, interface integrity. Capability
  targets attach only after the mechanism exists end to end and its instrument is non-vacuous.
- **Process:** labs never ask the owner to choose between arbitrary test numbers. A lab that
  wants a target proposes the instrument fix first; the PM rules on vacuity with evidence; the
  owner's role is limited to releasing holds and stopping mechanisms (D22).

## R15 — The D4 owner-release record is authored by the owner personally

**Authority, October 14:** owner decision in Claude's session, resolving C1 on PR #2421
(recorded at [#2421 comment 6159895665](https://github.com/UOR-Foundation/uor-r4/pull/2421#issuecomment-6159895665)).

- **No lab ever writes, edits, copies or generates `owner.json` (the D4 release record), or
  any file that stands in for it.** Casey creates it himself, from his own terminal, when and
  only when he decides to release the pilot.
- The gate is operator-attested by this rule, not cryptographically authenticated; the owner
  accepts that because all labs share his GitHub account and machine.
- The release tooling fills in every pinned identity from the sheet and takes only the owner's
  release decision as input; the exact single command is documented in the controller's
  `--help` and module docs (Codex, #2421), which also state that the owner record must be
  authored by the owner.
- A release record not created by the owner personally is void; any compute started from one
  is unauthorized.

**Amendment (D33, October 14):** for the pinned rung-0 A2 seed-1 diagnostic pilot only,
the owner delegated *execution* of `owner-release` to the Claude lab, conditional on Claude's
completed exact-head review and posted recommendation (satisfied by [#2421 comment
6160178151](https://github.com/UOR-Foundation/uor-r4/pull/2421#issuecomment-6160178151)).
The release *decision* remains Casey's; the delegate is named (`UOR_LAB=claude`), scoped to
the one pinned plan, and still bound by the typed confirmation phrase and every pin. The
record format is unchanged (`owner: Casey`, `owner-terminal:` receipt channel); the
delegation is attested by this amendment, D33, and the GitHub receipt chain, so the pinned
admission path and the qualified binary are untouched. The delegated run uses a tooling
build of the discrete crate whose only delta from the pinned driver is the actor guard; it
authors metadata only and starts no model computation. No other lab may create the record,
and this sets no precedent: every other hold stays owner-personal unless the owner
delegates again explicitly.
