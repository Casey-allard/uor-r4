# Codex worktree preservation — October 15, 2026

Owner-requested post-merge cleanup and retention of stale Codex workspaces.
This is source/document preservation, not a scientific experiment. No model
computation, training, GPU access or rental occurs. Training and M2 remain HELD.
Line: repository maintenance; research counts and milestone headlines unchanged.
Decision: retain inactive; no mechanism verdict or capability promotion.

## D-A delivery

Fresh `origin/main` contains #2508 merge
`abdf431edf80a80851d0d10a767886ae22502a6c`. Both the instrument directory and
`crates/uor-r4-discrete` are byte-identical to Claude-approved head
`8ec3bd7c84132f03916868b1f177d9bc3a70175b`.
The remote/local `codex/stage-a-eval-kept` branch and its worktree were removed.
Local non-build receipts were uploaded and download-hash verified as
`icloud:UOR-R4/results/codex/codex-stage-a-delivery-20261015.tar`, 113152 bytes,
MD5 `68d8c8f1d5eb612045d0f3be11dba393`; regenerable build caches were removed.
[Cleanup receipt](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6194795282).

`scripts/storage/uor-hygiene --apply` completed through a scope wrapper:
the worktree inventory includes only the active Codex cleanup tree; shared
cache/temp/result cleanup and global worktree pruning are excluded. This
honors the owner's instruction to leave other labs' branches and worktrees
alone. The wrapper is retained with the preservation archive. The script's
`uv cache prune` success line is its generic message; the wrapper deliberately
skips that shared-cache operation. The owner's checkout files stay unchanged.

## Dispositions and restoration

The two patches and exact base/result tree identities are indexed in the
[branch archive](../../history/branch-archive/INDEX.md#october-15--stale-codex-worktrees-retained-inactive).

- `codex-geometric-path-repair`: preserve all 12 uncommitted tracked changes,
  including `whole_path_law.rs`. This October 12 repair replaced several
  probability-law sites but still uses floating-point tensor/gradient paths.
  It is unfinished and inactive under D32. The original pause record says an
  earlier library check passed, later edits remained unqualified, and focused
  tests were stopped during compilation: NOT_RUN, not a negative. No new
  compilation or model test is performed on this historical source.
- `codex-learning-sheet-review-20261014`: preserve its sole tracked change,
  seven lines adding its own historical claim. This is not a pending model
  implementation. Preserve both local review directories, including findings,
  amendments, confirmations and the prior cloud receipt. Restore the claim
  only for historical examination, never into the current ownership store.
- `codex-harness-main`: detached at
  `837306f5335c0b9c5824256ac02c91dcde353577`, clean including ignored-file
  inventory, and reachable from fresh `origin/main`; no unique source or local
  material requires a patch. Remove after the preservation delivery.

The current repository is shallow: absence of ancestry for the October 12
base is not evidence of new branch work. Full `git archive` base snapshots
are retained so restoration does not depend on fetching old commit ancestry.
Base archive SHA256 values:

| Archive | SHA256 |
|---|---|
| `geometric/base-source.tar.gz` | `475b82214b2264edd723c0642bd1f2069f5fef8fdbf8bc3c823a35b8577dd365` |
| `learning/base-source.tar.gz` | `194bea24c45e88c1dfc539f412c703c9de7a4e2540a48622422360cd6a3feb04` |

The preservation archive contains those two base snapshots, both indexed
patches, the original geometric local directory, both learning-review record
directories, and the restoration checker/output. Regenerable Cargo caches are
excluded. Upload and download-hash verification completed for
`icloud:UOR-R4/results/codex/codex-stale-preservation-20261015.tar`,
902920704 bytes, MD5 `93ba6f9b39023f4d39ef31d5e2e67c4e`.
The earlier review archive remains separately indexed as
`discrete-learning-sheet-review-20261014`, MD5
`6e33578461ed50f71b21f5c03c12ce2a`; the new archive also retains the later
local confirmations rather than relying on that older copy.

Restore into a new inactive directory by extracting the corresponding base
archive, then applying its indexed patch with `git apply`. With the original
base commit available, `git read-tree BASE` and `git apply --cached PATCH`
under a separate `GIT_INDEX_FILE` reproduce the indexed result tree. Do not
apply either patch to current main or execute the archived model source.

## Verification and remaining dependency

Both patch applications pass `git apply --cached --check`; the reconstructed
full trees equal independently staged snapshots of the original dirty
worktrees. These checks use separate indexes and do not alter the originals.
The metadata diff passes whitespace checks. The geometric patch retains six
blank context lines represented by a single space; an unrestricted outer
`git diff --check` flags those patch-format lines. They are intentionally
preserved with the exact patch hash. D23/D25 ratchets, claim wording, state
formats and append-only ledger checks pass. The full harness check reports
zero errors with existing peer-state, old unattributed-receipt and unready
prep/panel warnings. No peer state is changed to silence those warnings.
The completed Codex claims are released in this delivery to avoid leaving
another merged branch in the durable ownership store.
Cloud receipt verification, source archive identities and source/document
checks are preservation evidence only. A protected merge must precede removal
of either dirty workspace; compare their current diffs to the archived bytes
again immediately before removal.

Codex remains D-A instrument-fault standby on #2029. Kimi owns the Stage-A
evaluation packet/run. A K8 reproduction, monotonicity or aggregate refusal is
INSTRUMENT_FAULT under D31, not a model finding. Codex may diagnose sealed
records and deliver a source-only fix with Claude exact-source review; no GPU
access follows without Kimi's explicit handoff.

**Next:** protected preservation delivery, verify on fresh `origin/main`, then
remove the named Codex worktrees/branches and publish the final #2029 receipt.
