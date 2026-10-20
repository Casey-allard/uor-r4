# D30 check-port 4 of 10: the `status` rules about the checkout

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; `check` module 4 of 10

## The two rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `status.branch_claimed` | warn | **1 warn** | **1 warn** |
| `status.no_orphan_branches` | warn | **2 warn** | **2 warn** |

Coverage now reads **16 of 40 rules** (harness 3, multilab 6, prep 5, status 2).

**These two are the `status` rules about *what this working copy is holding*,** which is exactly what
the port can verify offline. They also cover the two ways a checkout accumulates state the project says
it should not: an **unclaimed branch** and an **orphan branch**.

## Why the other two `status` rules are not here

`status.heartbeat_receipts` ("every in-progress milestone posts a status card") and
`status.registry_vs_github` ("the registry headline agrees with the newest status card") both read
**status cards posted as issue comments**. **They are the only two rules in `check` whose subject is the
GitHub conversation rather than the checkout**, and porting them means porting `gh` pagination, comment
parsing and the headline grammar together.

**That is a unit of work, not a line.** Riding it along here would produce two rules that *look* ported
and cannot be verified — the failure this sequencing exists to avoid — so they stay delegated, named,
and next.

## The distinction each rule turns on

**`branch_claimed`:** **claims are how a branch becomes answerable.** A branch checked out with no claim
is work no lab can be asked about — another lab cannot tell whether it is active, abandoned, or
scratch, and the board will not show it. `main`/`master` are exempt because the shared checkout is not
work; it is the base every branch comes from.

**`no_orphan_branches`:** proof is **by content, not ancestry.** The project **squash-merges**, and a
squash merge leaves no ancestry link — so an ancestry check would call a merged branch *unmerged* and
this rule would **never fire on the merges the project actually performs.** `githelp::merge_state` does
the content comparison, and the rule reports only `Merged`. **An `Unknown` branch is left alone, because
a rule that guesses "merged" tells a lab to delete work that may be live.**

A branch that is **checked out** is not an orphan: it is in use. The rule reuses `githelp`'s own
`checked_out` flag rather than forming a second opinion about what "in use" means — two definitions of
one rule is the defect class this line keeps producing.

## Verification

| check | result |
|---|---|
| Rust vs Python, both rules | **1 = 1 warn, 2 = 2 warn** |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **250 passed** (7 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `parity.py` | exit 0, **16 implemented** |

## Limitations, stated

- **`check` still delegates.** 4 of 10 modules; six remain.
- **The unmerged direction is the one that must never be guessed**, and the rule reports only on
  positive evidence of merge. That means **a genuinely orphaned branch whose content comparison returns
  `Unknown` is silently kept** — the safe direction, but it is a miss, not a pass.
- **`no_orphan_branches` reports the local half only.** Python's rule also names **clean worktrees whose
  branch is merged**; this unit covers branches. The worktree half needs `githelp`'s worktree listing
  consolidated, and it is recorded rather than half-done.
- **The agreement measurement is a hand-run probe**, read and removed. **Nothing in CI compares per-rule
  finding counts.**
- **The fix text differs from Python's** (`uor-harness git clean-merged` vs `python3 … cli.py`). The
  finding counts agree; a script scraping the fix string would see different words — **and the Rust
  string is the one that will still be valid after the port.**
