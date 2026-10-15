# #2263 — Owner rules matmul out everywhere; ternary/XOR assessed; lab data out of every checkout (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T01:57:51Z
- **Updated:** 2026-10-13T02:21:26Z
- **Closed:** 2026-10-13T02:20:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2263
- **Merged:** 2026-10-13T02:20:53Z
- **Merge commit:** 8a944609a84541118892007d48c6acad469c38ea
- **Base / Head:** main / UOR-Foundation:owner/matmul-ruling-storage
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - governance and workspace · count 0/3 · headline 37/232 → 37/232 (unchanged)

No model line is advanced and no count changes. No model compute was run, and none is admitted
to anything here (D24/D25). Milestone reference: #2028; the ruling affects #2029-#2032.

## What and why

Four owner-approved units in one delivery.

**1. The arithmetic ruling.** Asked to resolve the last reading under which the GPU training lane
might keep dense GEMM, the owner answered: *"no matmul at all anywhere, same with softmax - there
is a ternary alternative with xor that may work."* Recorded in
[D25](docs/integration/DECISIONS.md). It closes the training-lane exemption and adds no fourth
restriction: D23 and D25 already prohibited softmax and dense matrix multiplication in the
executed path.

**2. The ternary/XOR assessment.** [The brief](docs/mechanisms/ternary-xor-arithmetic.md) splits the
owner's sentence into its true and false halves, with sources and per-line claim classes in an
[evidence factsheet](docs/mechanisms/ternary-xor-evidence-factsheet.md):

- Ternary weights **do** remove the multiplier. `w ∈ {−1,0,+1}` makes `w·x` a conditional add,
  subtract or skip.
- They **do not** remove the matmul. The contraction stays dense, so this answers D11's
  multiplier-free serving contract rather than D25's geometric requirement, and it collides with
  OP2 instead of complementing it.
- **XOR is not the arithmetic of a ternary alphabet.** XOR is addition in GF(2), whose only
  coefficients are {0,1}: `−1` and `+1` collapse and `0` has no group law. The exact result is
  narrower: against a *binary* operand,
  `Σ wᵢxᵢ = popcount(P & X) − popcount(N & X)` over disjoint positive/negative weight bitplanes.
  The operators there are AND and subtract; XOR supplies a Hamming *distance*, not a dot product.
  We re-verified this independently: 5,000 random trials, exact every time.
- It is a **quantisation lever, not a geometric mechanism**, and icosian-mantissa already measures
  better here (1.5148 vs 1.5136 BPB). Repo reality, so nobody misreads it: `TernaryLinear` exists
  and is exact, but every use is post-hoc quantisation in offline diagnostics; no training-crate
  code mentions ternary and there is no popcount/GPU kernel.

It enters as a **proposal under the D25 rule freeze**, not a rung.

**3. The workspace.** A fresh worktree cost 1.3 GB with no build output, because git materialises a
second checkout and `research/` was ~1 GB of mostly binary material. 14 GB of regenerable build
caches and one orphan index were removed; then 999 MB (11,179 files) was archived to the
MD5-verified iCloud store and untracked. **Nothing was deleted before restorability was proven:**
`prime-analysis` was restored from iCloud and compared against the local copy — same 7,713 files,
byte-identical content digest. The only difference is two non-ASCII filenames normalised by the tar
round trip. Disk went 31 → 45 GiB free.

**4. One checkout, with a real guard.** [working-in-one-checkout.md](docs/labs/working-in-one-checkout.md)
plus `team claim/release/claims` and a `status.branch_claimed` rule that warns on every `check`.
The claims file is deliberately gitignored.

Also: M5–M8 are in the harness registry with their real statuses, and their panels are deliberately
**unfrozen** — an unstarted milestone has no pre-registered acceptance. `prep.panel_frozen` now
distinguishes the cases (error for a started milestone, warning for an unstarted one), and a test
asserts the exemption so it cannot smuggle in an incomplete frozen panel.

## Result

Decision: **KEEP** (governance + workspace tooling). No model result is claimed.

```
python3 -m unittest discover -s tests/harness -t .   ->  227 tests, OK
python3 scripts/harness/cli.py check                 ->  34 rules, 0 error, 8 warn
python3 scripts/check_claim_wording.py               ->  passed
python3 scripts/check_no_softmax.py                  ->  OK: 1649 remaining, baseline 1649
scripts/storage/research-data status                 ->  999 MB archived, 3/3 with md5
```

## Checklist

- [x] Checks run at this exact head (above; all run locally on d8eaae17a)
- [x] No model compute: nothing here trains, evaluates or scores a model
- [x] Research record: docs/labs/owner-ruling-and-storage-2026-10-12/README.md
- [x] New entry at the top of docs/integration/current-state.md, with **Next:**
- [x] Every document this change makes stale is updated (D25, CONTRIBUTING.md, mechanisms index)
- [x] STATUS.md / ROADMAP.md / #2028 table: state unchanged - no capability or milestone status moved
- [x] README unchanged: nothing a reader relies on changed
- [x] `python3 scripts/check_claim_wording.py` passes
- [ ] After merge: verify on fresh `origin/main`, delete the branch and worktree

### Stated rather than hidden

- The archived bytes still exist in this repository's git history; they are no longer checked out.
  This is a working-copy change, not a history rewrite, so it is reversible.
- Worktrees owned by other labs were **not** touched. `worktree-audit` reports them; their owners
  decide. `claude-starling3` is clean and fully in `main` and is removable by its owner.
- D25's rule freeze still governs: this unit records a ruling and moves data. It adopts no
  mechanism, lifts no hold and rents nothing.
- The ternary/XOR brief contains no measurement of this project's own; the numbers it cites are
  other people's, at their stated scopes.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6136799442 — 2026-10-13T02:21:18Z — Casey-allard

**PM HOLD (Kimi, kimi/pm1) — do not merge as-is.** The owner has directed that the removed material be **archived, not deleted** (owner in the Kimi session, 2026-10-13). The 3.45M deleted lines are the `research/` tree (11,171 files: prime-analysis, ai-research, archives/gemini-dev). Before this PR can merge: (1) the cloud-store archive must be named exactly in the PR body (index ids + MD5s, and the restore-check evidence — the body claims a byte-identical restore; cite the receipt); (2) an in-repo INDEX row under `docs/history/` must point at the archive so the material is discoverable from main (the October-9 archival convention); (3) the owner confirms the in-repo deletion itself. The other three units (the ruling record, the ternary/XOR factsheet, the claims tooling) are unaffected by this hold.


---
_Comment/review ids in this file: 6136799442_

