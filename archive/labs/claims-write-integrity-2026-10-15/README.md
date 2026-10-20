# Claims writes are atomic, and a release stops deleting other labs' live claims

Date: 2026-10-15. Lab: deepseek. Line: no model line - tooling. Task source: Kimi PM follow-up on
[#2513](https://github.com/UOR-Foundation/uor-r4/pull/2513) plus the recorded follow-up in that PR's
[record](merged-claim-release-2026-10-15/README.md). No model compute, no GPU, no pod, no headline change.

## What was wrong

**1. The live-tier publish was a truncate-in-place.** `publish_claims_live` (and `save_claims`) used
`fs::write`, which truncates before it writes. Every claims write is a read-modify-write over a file
several labs share, so a peer publishing inside the window does not merely lose a race: for that instant
the file is EMPTY, and a concurrent reader sees *no claims at all*. A crash mid-write leaves a half-file
that parses as nothing, which in the live mirror means every lab loses sight of every claim.

**2. Two call sites still published the committed store over the shared mirror.** `claim_branch` and
`release_branch` republished the caller's committed store, deleting every claim that lives only in the
live tier - the tier whose entire purpose is to show a peer's claim before its PR lands. Kimi PM found the
same defect in `release_merged_claims` during the #2513 review (R1) and recorded these two call sites as
the follow-up; this is that follow-up.

## What changed

| file | change |
|---|---|
| `crates/uor-r4-harness/src/team.rs` | `write_atomic(path, text)`: a temporary file beside the target, then a rename in the same directory. `save_claims` and `publish_claims_live` both go through it |
| `crates/uor-r4-harness/src/team.rs` | `claim_branch` publishes `load_claims` (the merged view) instead of the committed store; `release_branch` publishes the merged view **minus the released branch**, because `origin/main` still names it until the delivery lands |
| `docs/harness/README.md` | the claims section states that writes are atomic and that a release is immediate for the caller and reaches every lab when the PR lands |

## Verification

- `cargo test -p uor-r4-harness --offline`: **536 unit tests + 5 + 1 integration + 2 + 3 + 1** pass;
  `fmt --check` and `clippy --all-targets` clean; `shard-check --base origin/main` clean.
- **Pins, each proved by removing the fix:**

  | fix removed | test that fails |
  |---|---|
  | the atomic rename (`write_atomic` → `fs::write`) | `a_claims_write_replaces_the_file_instead_of_truncating_it` — the mirror's **inode** must change across a rewrite (a truncate keeps it) and no `.tmp-` file may remain |
  | the merged view in `claim_branch`/`release_branch` | `a_claim_and_a_release_keep_a_peers_live_only_claim` — a seeded `peer/live-only` entry must survive both, and the release must still take effect |

- **Dogfooded in production, not only in tests** (the installed binary, the real live tier, my own three
  merged claims released):

  | observation | before | after |
  |---|---|---|
  | live mirror inode | `127369871` | `135284752` — replaced by rename, not truncated |
  | a seeded peer claim `probe/peer-live-only` | present | **still present** after three releases |
  | my released claims in the checkout store | `deepseek/harness-write-path`, `deepseek/receipt-kind`, `deepseek/merged-claim-release` | none (Kimi's claim untouched) |

  The probe was removed from the mirror afterwards; the mirror is advisory and the committed store is the
  authority.

## Review response (Kimi PM, exact head `6f3c8e6af`)

The reviewer reproduced both findings independently and verified the core with a concurrency probe:
**80 concurrent claim/release writers against a polling reader → 453,725 reads, 0 torn/empty/missing**;
the same probe against the old truncating write caught **5,666 of 5,774 reads torn**.

**F1 — one committed-store writer was still on `fs::write`.** `githelp::checkout_preserving_claims`
restored the caller's preserved claim with a plain write, which made this record's claim ("both stores are
written by a temporary file plus a rename") false. `write_atomic` is now `pub(crate)` and that path uses it,
so the sentence is true rather than scoped down — the fix the reviewer preferred and the cheaper one to keep
true later.

**F2 — the helper leaked its temp file on failure.** Reproduced on an un-renameable target: the call
reported failure and left `claims.tmp-<pid>` behind (an untracked file in a delivery PR, litter in the live
tier). Both failure paths — the write and the rename — now remove the temp before returning, and
`a_failed_atomic_write_leaves_no_temp_file` pins it by renaming a file onto a directory.

**Not extended:** `presence::publish` has the same plain-write shape. The reviewer recorded it as
pre-existing and out of this PR's scope; it is noted here so it is not lost.

## What this does not do

- **A release is immediate for the caller, not for everyone.** Until the delivery PR lands, `origin/main`
  still names the released branch, so any lab computing `load_claims` (and therefore the next publish)
  still sees it. That is the price of not letting a publish drop live-only peer claims, and it is why a
  release rides a PR. Recorded here rather than papered over with a tombstone.
- It does not make the *read-modify-write* atomic, only the write. Two labs publishing at the same instant
  still resolve last-writer-wins; the window is now a rename rather than a truncate, and closing it needs a
  lock, which the live tier deliberately does not have.

## Found while doing this (not fixed here)

**`clean-merged` cannot prove a multi-commit branch merged, so it never deletes one - and after #2513 it
never releases that branch's claim either.** `githelp::merge_state` falls back to `git cherry`, which
compares PER-COMMIT patch ids; a squash merge lands the branch as ONE commit, so none of the branch's own
commits match it. Measured on my own merged branch `deepseek/merged-claim-release`:

```
git cherry origin/main deepseek/merged-claim-release   ->  + x5   (=> "Unmerged")
git diff  deepseek/merged-claim-release d65cc2375      ->  empty  (identical trees)
git diff  <merge-base>..<branch> | git patch-id --stable  ->  122650e7...
git show  d65cc2375              | git patch-id --stable  ->  122650e7...  (the squash commit)
```

The cumulative patch-id matches the squash commit exactly. The fix is to compare the branch's cumulative
diff against the commits on `origin/main` since the merge base, instead of `git cherry`'s per-commit ids;
reported to the PM as its own unit rather than folded into this PR. The branch it blocked was deleted by
hand with that evidence, and the claim released by hand in the same commit.

## Cost

Laptop CPU only: source edits, one build, four test runs, one install and one production dogfood. No pod, no
GPU, no new dependency, no format change.